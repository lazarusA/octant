//! The browser's in-memory store, which also holds parts of objects: of a
//! shard, its index and the inner chunks fetched so far (`shard_fetch.rs`).
//! Range reads are answered from the parts held; whole objects behave as in
//! a `MemoryStore`.

use std::collections::HashMap;
use std::sync::RwLock;

use zarrs::storage::byte_range::{ByteRange, ByteRangeIterator};
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{
    Bytes, ListableStorageTraits, MaybeBytesIterator, ReadableStorageTraits, StorageError,
    StoreKey, StoreKeys, StoreKeysPrefixes, StorePrefix, WritableStorageTraits,
};

/// Parts of an object of `size` bytes: `(offset, bytes)`, in no order.
#[derive(Default)]
struct Parts {
    size: u64,
    pieces: Vec<(u64, Bytes)>,
}

impl Parts {
    /// The bytes of `range` (of `self.size`) when one piece holds them
    /// (zarrs reads each inner chunk whole, within one fetched range).
    fn read(&self, range: ByteRange) -> Option<Bytes> {
        let want = range.to_range(self.size);
        self.pieces.iter().find_map(|(offset, bytes)| {
            let end = offset + bytes.len() as u64;
            (want.start >= *offset && want.end <= end).then(|| {
                let from = (want.start - offset) as usize;
                bytes.slice(from..from + (want.end - want.start) as usize)
            })
        })
    }

    /// Whether `start..end` lies within one held piece.
    fn holds(&self, start: u64, end: u64) -> bool {
        self.pieces
            .iter()
            .any(|(o, b)| start >= *o && end <= o + b.len() as u64)
    }
}

/// A `MemoryStore` that also holds parts of objects.
#[derive(Default)]
pub struct PartialStore {
    whole: MemoryStore,
    parts: RwLock<HashMap<StoreKey, Parts>>,
}

impl PartialStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `bytes` at `offset` of the object `key` of `size` bytes, unless
    /// a piece already holds them (two loads fetched the same range).
    pub fn insert_part(&self, key: &StoreKey, size: u64, offset: u64, bytes: Bytes) {
        let mut parts = self.parts.write().unwrap_or_else(|p| p.into_inner());
        let entry = parts.entry(key.clone()).or_default();
        entry.size = size;
        if !entry.holds(offset, offset + bytes.len() as u64) {
            entry.pieces.push((offset, bytes));
        }
    }

    /// Whether bytes `start..end` of the object `key` are held.
    pub fn holds(&self, key: &StoreKey, start: u64, end: u64) -> bool {
        if matches!(self.whole.size_key(key), Ok(Some(_))) {
            return true;
        }
        let parts = self.parts.read().unwrap_or_else(|p| p.into_inner());
        parts.get(key).is_some_and(|p| p.holds(start, end))
    }
}

impl ReadableStorageTraits for PartialStore {
    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        if matches!(self.whole.size_key(key), Ok(Some(_))) {
            return self.whole.get_partial_many(key, byte_ranges);
        }
        let parts = self.parts.read().unwrap_or_else(|p| p.into_inner());
        let Some(held) = parts.get(key) else {
            return Ok(None);
        };
        let read: Vec<Result<Bytes, StorageError>> = byte_ranges
            .map(|range| {
                held.read(range).ok_or_else(|| {
                    StorageError::Other(format!("bytes {range:?} of '{key}' were not fetched"))
                })
            })
            .collect();
        Ok(Some(Box::new(read.into_iter())))
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        if let Some(size) = self.whole.size_key(key)? {
            return Ok(Some(size));
        }
        let parts = self.parts.read().unwrap_or_else(|p| p.into_inner());
        Ok(parts.get(key).map(|p| p.size))
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

impl WritableStorageTraits for PartialStore {
    fn set(&self, key: &StoreKey, value: Bytes) -> Result<(), StorageError> {
        // The whole object replaces any parts of it.
        let mut parts = self.parts.write().unwrap_or_else(|p| p.into_inner());
        parts.remove(key);
        self.whole.set(key, value)
    }

    fn set_partial_many(
        &self,
        _key: &StoreKey,
        _offset_values: zarrs::storage::OffsetBytesIterator<Bytes>,
    ) -> Result<(), StorageError> {
        Err(StorageError::Other(
            "partial writes are not supported".into(),
        ))
    }

    fn erase(&self, key: &StoreKey) -> Result<(), StorageError> {
        let mut parts = self.parts.write().unwrap_or_else(|p| p.into_inner());
        parts.remove(key);
        self.whole.erase(key)
    }

    fn erase_prefix(&self, prefix: &StorePrefix) -> Result<(), StorageError> {
        let mut parts = self.parts.write().unwrap_or_else(|p| p.into_inner());
        parts.retain(|k, _| !k.as_str().starts_with(prefix.as_str()));
        self.whole.erase_prefix(prefix)
    }

    fn supports_set_partial(&self) -> bool {
        false
    }
}

impl ListableStorageTraits for PartialStore {
    fn list(&self) -> Result<StoreKeys, StorageError> {
        self.whole.list()
    }

    fn list_prefix(&self, prefix: &StorePrefix) -> Result<StoreKeys, StorageError> {
        self.whole.list_prefix(prefix)
    }

    fn list_dir(&self, prefix: &StorePrefix) -> Result<StoreKeysPrefixes, StorageError> {
        self.whole.list_dir(prefix)
    }

    fn size_prefix(&self, prefix: &StorePrefix) -> Result<u64, StorageError> {
        self.whole.size_prefix(prefix)
    }
}
