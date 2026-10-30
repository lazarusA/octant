//! A read that fails (a timeout, a server error) is not a missing document:
//! the store's shared cache never remembers it, so a later reader asks again.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use zarrs::storage::byte_range::ByteRangeIterator;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{
    MaybeBytes, MaybeBytesIterator, ReadableStorageTraits, StorageError, StoreKey,
    WritableStorageTraits,
};

use super::{MetaCache, MetaReader};

const GROUP: &str = "g/zarr.json";

/// Group `g` with one attribute, whose reads fail while `failing`.
struct Flaky {
    inner: MemoryStore,
    failing: AtomicBool,
}

impl Flaky {
    fn new() -> Arc<Self> {
        let inner = MemoryStore::new();
        let doc = r#"{"zarr_format":3,"node_type":"group","attributes":{"source":"g"}}"#;
        let key = StoreKey::new(GROUP).expect("key");
        inner
            .set(&key, doc.as_bytes().to_vec().into())
            .expect("set");
        Arc::new(Self {
            inner,
            failing: AtomicBool::new(true),
        })
    }

    fn fails(&self, key: &StoreKey) -> Result<(), StorageError> {
        if self.failing.load(Ordering::SeqCst) && key.as_str() == GROUP {
            return Err(StorageError::Other("timed out".into()));
        }
        Ok(())
    }
}

impl ReadableStorageTraits for Flaky {
    fn get(&self, key: &StoreKey) -> Result<MaybeBytes, StorageError> {
        self.fails(key)?;
        self.inner.get(key)
    }

    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        self.fails(key)?;
        self.inner.get_partial_many(key, byte_ranges)
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        self.inner.size_key(key)
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

fn source(attrs: &super::meta_cache::GroupAttrs) -> Option<&str> {
    attrs.as_ref()?.get("source")?.as_str()
}

#[test]
fn a_failed_read_is_asked_again_by_the_next_reader() {
    let store = Flaky::new();
    let cache = MetaCache::shared();
    let reader = MetaReader::direct(store.clone()).with_cache(&cache);
    assert_eq!(reader.group_attributes("g"), Ok(None));
    assert_eq!(cache.group("g"), None, "a failure is not remembered");

    store.failing.store(false, Ordering::SeqCst);
    let reader = MetaReader::direct(store.clone()).with_cache(&cache);
    let attrs = reader.group_attributes("g").expect("read");
    assert_eq!(source(&attrs), Some("g"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_walk_does_not_remember_a_failed_read() {
    let store = Flaky::new();
    let cache = MetaCache::shared();
    let walk = |cache: &MetaCache| {
        super::direct::walk_concurrent(store.clone(), cache, 3, |reader| {
            reader.group_attributes("g").ok()
        })
    };
    assert_eq!(walk(&cache), Some(None));
    assert_eq!(cache.group("g"), None, "a failure is not remembered");

    store.failing.store(false, Ordering::SeqCst);
    let attrs = walk(&cache).expect("settled");
    assert_eq!(source(&attrs), Some("g"));
}
