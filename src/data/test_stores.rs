//! Test stores whose reads fail: a timeout or a server error, or S3's 403
//! for a missing key where it may not list.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use zarrs::storage::byte_range::ByteRangeIterator;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{
    ListableStorageTraits, MaybeBytes, MaybeBytesIterator, ReadableStorageTraits, StorageError,
    StoreKey, StoreKeys, StoreKeysPrefixes, StorePrefix, WritableStorageTraits,
};

/// `inner` whose reads of the keys `fails` picks are errors while `failing`,
/// counting the reads of the root `zarr.json`.
pub struct FailingStore {
    inner: Arc<MemoryStore>,
    pub failing: AtomicBool,
    fails: fn(&StoreKey, &MemoryStore) -> bool,
    pub root_reads: AtomicUsize,
}

impl FailingStore {
    pub fn new(inner: Arc<MemoryStore>, fails: fn(&StoreKey, &MemoryStore) -> bool) -> Arc<Self> {
        Arc::new(Self {
            inner,
            failing: AtomicBool::new(true),
            fails,
            root_reads: AtomicUsize::new(0),
        })
    }

    /// Fails reads of the root `zarr.json`.
    pub fn root(key: &StoreKey, _: &MemoryStore) -> bool {
        key.as_str() == "zarr.json"
    }

    /// Fails reads of a missing `zarr.json`, as S3 answers 403.
    pub fn missing_v3(key: &StoreKey, inner: &MemoryStore) -> bool {
        key.as_str().ends_with("zarr.json") && matches!(inner.get(key), Ok(None))
    }

    fn check(&self, key: &StoreKey) -> Result<(), StorageError> {
        if key.as_str() == "zarr.json" {
            self.root_reads.fetch_add(1, Ordering::SeqCst);
        }
        match self.failing.load(Ordering::SeqCst) && (self.fails)(key, &self.inner) {
            true => Err(StorageError::Other("timed out".into())),
            false => Ok(()),
        }
    }
}

impl ReadableStorageTraits for FailingStore {
    fn get(&self, key: &StoreKey) -> Result<MaybeBytes, StorageError> {
        self.check(key)?;
        self.inner.get(key)
    }

    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        self.check(key)?;
        self.inner.get_partial_many(key, byte_ranges)
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        self.inner.size_key(key)
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

impl ListableStorageTraits for FailingStore {
    fn list(&self) -> Result<StoreKeys, StorageError> {
        self.inner.list()
    }

    fn list_prefix(&self, prefix: &StorePrefix) -> Result<StoreKeys, StorageError> {
        self.inner.list_prefix(prefix)
    }

    fn list_dir(&self, prefix: &StorePrefix) -> Result<StoreKeysPrefixes, StorageError> {
        self.inner.list_dir(prefix)
    }

    fn size_prefix(&self, prefix: &StorePrefix) -> Result<u64, StorageError> {
        self.inner.size_prefix(prefix)
    }
}

impl WritableStorageTraits for FailingStore {
    fn set(&self, key: &StoreKey, value: zarrs::storage::Bytes) -> Result<(), StorageError> {
        self.inner.set(key, value)
    }

    fn set_partial_many(
        &self,
        key: &StoreKey,
        offset_values: zarrs::storage::OffsetBytesIterator,
    ) -> Result<(), StorageError> {
        self.inner.set_partial_many(key, offset_values)
    }

    fn erase(&self, key: &StoreKey) -> Result<(), StorageError> {
        self.inner.erase(key)
    }

    fn erase_prefix(&self, prefix: &StorePrefix) -> Result<(), StorageError> {
        self.inner.erase_prefix(prefix)
    }

    fn supports_set_partial(&self) -> bool {
        false
    }
}
