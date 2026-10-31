//! A read that fails (a timeout, a server error) is not a missing document:
//! the store's shared cache never remembers it, so a later reader asks again.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

use super::{MetaCache, MetaReader};
use crate::data::test_stores::FailingStore;

const GROUP: &str = "g/zarr.json";

/// Group `g` with one attribute, whose reads fail while `failing`.
fn flaky() -> Arc<FailingStore> {
    let inner = MemoryStore::new();
    let doc = r#"{"zarr_format":3,"node_type":"group","attributes":{"source":"g"}}"#;
    let key = StoreKey::new(GROUP).expect("key");
    inner
        .set(&key, doc.as_bytes().to_vec().into())
        .expect("set");
    FailingStore::new(Arc::new(inner), |key, _| key.as_str() == GROUP)
}

fn source(attrs: &super::meta_cache::GroupAttrs) -> Option<&str> {
    attrs.as_ref()?.get("source")?.as_str()
}

#[test]
fn a_failed_read_is_asked_again_by_the_next_reader() {
    let store = flaky();
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
    let store = flaky();
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
