//! The memory budget of downloaded chunks: least recently used objects are
//! dropped across stores once past it, never those a lease holds, nor
//! metadata.

use std::sync::Arc;

use zarrs::storage::{Bytes, ReadableStorageTraits, StoreKey, WritableStorageTraits};

use super::partial_store::PartialStore;
use crate::data::blocks::memory_budget::MemoryBudget;

const KB: u64 = 1024;

fn key(k: &str) -> StoreKey {
    StoreKey::new(k).expect("key")
}

fn bytes(n: u64) -> Bytes {
    Bytes::from(vec![1u8; n as usize])
}

/// A budget of 10 KiB, trimmed to 6 KiB.
fn budget() -> Arc<MemoryBudget> {
    MemoryBudget::new(10 * KB, 6 * KB)
}

fn has(store: &PartialStore, k: &str) -> bool {
    matches!(store.size_key(&key(k)), Ok(Some(_)))
}

#[test]
fn the_least_recently_used_go_first_once_past_the_budget() {
    let store = PartialStore::with_budget(budget());
    for i in 0..5 {
        store
            .insert_chunk(&key(&format!("v/c/{i}")), bytes(2 * KB))
            .expect("insert");
    }
    assert!(
        (0..5).all(|i| has(&store, &format!("v/c/{i}"))),
        "at the budget"
    );
    store.insert_part(&key("v/c/5"), 100 * KB, 0, bytes(2 * KB));
    // 12 KiB held: the oldest go until 6 KiB are left.
    assert!(!has(&store, "v/c/0") && !has(&store, "v/c/1") && !has(&store, "v/c/2"));
    assert!(has(&store, "v/c/3") && has(&store, "v/c/4") && has(&store, "v/c/5"));
}

#[test]
fn leased_objects_and_metadata_stay() {
    let budget = budget();
    let store = PartialStore::with_budget(budget.clone());
    store
        .set(&key("v/zarr.json"), bytes(20 * KB))
        .expect("metadata");
    store.insert_chunk(&key("v/c/0"), bytes(4 * KB)).expect("0");
    store.insert_chunk(&key("x/c/0"), bytes(4 * KB)).expect("x");
    let lease = store.lease(vec![key("v/c/0")], vec!["x/".into()]);
    store.insert_chunk(&key("v/c/1"), bytes(4 * KB)).expect("1");
    assert!(has(&store, "v/zarr.json"), "metadata is not counted");
    assert!(has(&store, "v/c/0") && has(&store, "x/c/0"), "leased");
    assert!(!has(&store, "v/c/1"), "the only object free to go");
    drop(lease);
    // Released within the budget: they stay until it is passed again.
    assert_eq!(budget.held(), 8 * KB);
    store.insert_chunk(&key("v/c/2"), bytes(4 * KB)).expect("2");
    assert!(!has(&store, "v/c/0") && !has(&store, "x/c/0"));
    assert!(has(&store, "v/c/2"));
}

#[test]
fn one_budget_trims_every_store() {
    let budget = budget();
    let a = PartialStore::with_budget(budget.clone());
    let b = PartialStore::with_budget(budget.clone());
    a.insert_chunk(&key("v/c/0"), bytes(6 * KB)).expect("a");
    b.insert_chunk(&key("v/c/0"), bytes(6 * KB)).expect("b");
    assert!(!has(&a, "v/c/0"), "the older store's chunk goes");
    assert!(has(&b, "v/c/0"));
    assert_eq!(budget.held(), 6 * KB);
}

#[test]
fn replacing_and_erasing_release_their_bytes() {
    let budget = budget();
    let store = PartialStore::with_budget(budget.clone());
    store.insert_part(&key("v/c/0"), 10 * KB, 0, bytes(3 * KB));
    store.insert_part(&key("v/c/0"), 10 * KB, 0, bytes(3 * KB));
    assert_eq!(budget.held(), 3 * KB, "a piece already held is not added");
    store.set(&key("v/c/0"), bytes(KB)).expect("set");
    assert_eq!(budget.held(), 0, "a kept whole object replaces the parts");
    store.insert_chunk(&key("v/c/1"), bytes(2 * KB)).expect("1");
    store.insert_chunk(&key("w/c/0"), bytes(2 * KB)).expect("w");
    store
        .erase_prefix(&zarrs::storage::StorePrefix::new("v/").expect("prefix"))
        .expect("erase");
    assert_eq!(budget.held(), 2 * KB);
}

#[test]
fn a_dropped_store_no_longer_counts() {
    let budget = budget();
    let store = PartialStore::with_budget(budget.clone());
    store.insert_chunk(&key("v/c/0"), bytes(4 * KB)).expect("0");
    store.insert_part(&key("v/c/1"), 10 * KB, 0, bytes(2 * KB));
    drop(store);
    assert_eq!(budget.held(), 0);
}

#[test]
fn a_released_url_opens_a_fresh_store() {
    use super::WasmZarrBlockStore;
    let url = "memory://released/";
    let open = WasmZarrBlockStore::get_or_create(url);
    assert!(Arc::ptr_eq(&open, &WasmZarrBlockStore::get_or_create(url)));
    WasmZarrBlockStore::release(url);
    assert!(!Arc::ptr_eq(&open, &WasmZarrBlockStore::get_or_create(url)));
}

#[test]
fn a_trim_that_frees_nothing_waits_for_a_release() {
    let budget = budget();
    let store = PartialStore::with_budget(budget.clone());
    let lease = store.lease(vec![key("v/c/0"), key("v/c/1")], Vec::new());
    store.insert_chunk(&key("v/c/0"), bytes(6 * KB)).expect("0");
    store.insert_chunk(&key("v/c/1"), bytes(6 * KB)).expect("1");
    // Leases alone are past the budget: a little more does not trim again.
    store.insert_chunk(&key("w/c/0"), bytes(KB)).expect("w");
    assert!(has(&store, "w/c/0"));
    drop(lease);
    // Released leases count as just used: the older chunk goes first, then
    // one of the two released together (they tie, in no set order).
    assert!(!has(&store, "w/c/0"));
    assert!(has(&store, "v/c/0") != has(&store, "v/c/1"));
    assert_eq!(budget.held(), 6 * KB);
}
