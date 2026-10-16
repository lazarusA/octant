//! The zarrs storage traits of [`PartialStore`].

use zarrs::storage::byte_range::ByteRangeIterator;
use zarrs::storage::{
    Bytes, ListableStorageTraits, MaybeBytesIterator, ReadableStorageTraits, StorageError,
    StoreKey, StoreKeys, StoreKeysPrefixes, StorePrefix, WritableStorageTraits,
};

use super::PartialStore;

impl ReadableStorageTraits for PartialStore {
    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        if matches!(self.whole.size_key(key), Ok(Some(_))) {
            return self.whole.get_partial_many(key, byte_ranges);
        }
        let held = self.held.read().unwrap_or_else(|p| p.into_inner());
        let Some(parts) = held.parts.get(key) else {
            return Ok(None);
        };
        let read: Vec<Result<Bytes, StorageError>> = byte_ranges
            .map(|range| {
                parts.read(range).ok_or_else(|| {
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
        let held = self.held.read().unwrap_or_else(|p| p.into_inner());
        Ok(held.parts.get(key).map(|p| p.size))
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

impl WritableStorageTraits for PartialStore {
    fn set(&self, key: &StoreKey, value: Bytes) -> Result<(), StorageError> {
        // The whole object replaces any parts of it, and is kept.
        let freed = self.held_mut().forget(key);
        self.budget.released(freed);
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
        let freed = self.held_mut().forget(key);
        self.budget.released(freed);
        self.whole.erase(key)
    }

    fn erase_prefix(&self, prefix: &StorePrefix) -> Result<(), StorageError> {
        let freed = {
            let mut guard = self.held_mut();
            let held = &mut *guard;
            let under = |k: &StoreKey| k.as_str().starts_with(prefix.as_str());
            held.parts.retain(|k, _| !under(k));
            let mut freed = 0;
            held.uses.retain(|k, used| {
                let keep = !under(k);
                if !keep {
                    freed += used.bytes;
                }
                keep
            });
            freed
        };
        self.budget.released(freed);
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
