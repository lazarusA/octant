//! Array opens read each group's metadata once per store, not once per open.

use std::sync::{Arc, Mutex};

use zarrs::array::{ArrayBuilder, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::storage_adapter::usage_log::UsageLogStorageAdapter;
use zarrs::storage::store::MemoryStore;

use super::generic::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

/// A root group holding the 4 x 4 `float32` arrays `a`, `b` and `c`.
fn store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    zarrs::group::GroupBuilder::new()
        .build(store.clone(), "/")
        .expect("group")
        .store_metadata()
        .expect("root");
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    for name in ["/a", "/b", "/c"] {
        let mut builder = ArrayBuilder::new(
            vec![4, 4],
            vec![4, 4],
            f32_dt.clone(),
            FillValue::from(0f32),
        );
        builder.dimension_names(Some(["y", "x"]));
        let array = builder.build(store.clone(), name).expect("build");
        array.store_metadata().expect("metadata");
    }
    store
}

/// `store()` behind a log of its reads, as a block store.
fn logged() -> (GenericZarrBlockStore, Arc<Mutex<Vec<u8>>>) {
    let log = Arc::new(Mutex::new(Vec::<u8>::new()));
    let logged = UsageLogStorageAdapter::new(store(), log.clone(), String::new);
    let store = GenericZarrBlockStore::new(Arc::new(logged), "memory://groups", "zarr", "Zarr");
    (store, log)
}

/// Reads of the root `zarr.json` so far, whole or partial (`get(zarr.json)`,
/// `get_partial_many(zarr.json, ..)`).
fn root_reads(log: &Mutex<Vec<u8>>) -> usize {
    let log = log.lock().unwrap_or_else(|p| p.into_inner());
    let text = String::from_utf8_lossy(&log);
    text.lines().filter(|l| l.contains("(zarr.json")).count()
}

/// The first block of `variable`, opening its array.
fn open(store: &GenericZarrBlockStore, variable: &str) {
    let request = SliceRequest {
        variable: variable.into(),
        selections: vec![DimensionSelection::Index(0), DimensionSelection::Index(0)],
    };
    store.fetch_block(&request).expect("block");
}

#[test]
fn array_opens_read_the_root_once() {
    let (store, log) = logged();
    let mut reads = Vec::new();
    for variable in ["a", "b", "c"] {
        open(&store, variable);
        reads.push(root_reads(&log));
    }
    // The first open reads the root; the others ask it nothing more.
    assert!(reads[0] > 0, "{reads:?}");
    assert_eq!(reads, [reads[0]; 3]);
}

#[test]
fn inspection_and_opens_share_the_root() {
    let (store, log) = logged();
    store.inspect().expect("inspect");
    for variable in ["a", "b", "c"] {
        open(&store, variable);
    }
    // The store opens the root group once, for its consolidated metadata;
    // OME discovery, the variable listing, pyramid links, grid mappings and
    // every array open take its attributes from there.
    assert_eq!(root_reads(&log), 1);
}

/// A Zarr v2 store: root and group `g` with attributes, array `g/a`.
fn v2_store() -> Arc<MemoryStore> {
    use zarrs::storage::{StoreKey, WritableStorageTraits};
    let store = Arc::new(MemoryStore::new());
    let array = r#"{"zarr_format":2,"shape":[4,4],"chunks":[4,4],"dtype":"<f4",
        "compressor":null,"fill_value":0,"order":"C","filters":null}"#;
    for (key, json) in [
        (".zgroup", r#"{"zarr_format":2}"#),
        (".zattrs", r#"{"title":"root"}"#),
        ("g/.zgroup", r#"{"zarr_format":2}"#),
        ("g/.zattrs", r#"{"source":"group"}"#),
        ("g/a/.zarray", array),
        ("g/a/.zattrs", r#"{"_ARRAY_DIMENSIONS":["y","x"]}"#),
    ] {
        let key = StoreKey::new(key).expect("key");
        store
            .set(&key, json.as_bytes().to_vec().into())
            .expect("set");
    }
    store
}

#[test]
fn a_v2_store_inherits_every_group() {
    let store = GenericZarrBlockStore::new(v2_store(), "memory://v2", "zarr", "Zarr");
    store.inspect().expect("inspect");
    let request = SliceRequest {
        variable: "g/a".into(),
        selections: vec![DimensionSelection::Index(0), DimensionSelection::Index(0)],
    };
    let block = store.fetch_block(&request).expect("block");
    // The root's attributes come from the group zarrs opened, the group's
    // from its `.zattrs`, asked in the format the root recorded.
    assert_eq!(
        block.attributes.get("title").map(String::as_str),
        Some("root")
    );
    assert_eq!(
        block.attributes.get("source").map(String::as_str),
        Some("group")
    );
}

#[test]
fn a_consolidated_store_lists_its_variables_without_listing_keys() {
    use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
    let memory = store();
    let node = zarrs::node::Node::open(&memory, "/").expect("node");
    let metadata = node.consolidate_metadata().expect("consolidated");
    let mut root = zarrs::group::Group::open(memory.clone(), "/").expect("root");
    root.set_consolidated_metadata(Some(ConsolidatedMetadata {
        metadata,
        ..ConsolidatedMetadata::default()
    }));
    root.store_metadata().expect("root");

    let log = Arc::new(Mutex::new(Vec::<u8>::new()));
    let logged = UsageLogStorageAdapter::new(memory, log.clone(), String::new);
    let store =
        GenericZarrBlockStore::new(Arc::new(logged), "memory://consolidated", "zarr", "Zarr");
    let meta = store.inspect().expect("inspect");
    let mut names: Vec<&str> = meta.variables.iter().map(|v| v.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, ["a", "b", "c"]);
    // Remote stores cannot be listed: the variables come from the root's
    // consolidated metadata alone.
    let log = log.lock().unwrap_or_else(|p| p.into_inner());
    let text = String::from_utf8_lossy(&log);
    assert!(!text.lines().any(|l| l.starts_with("list")), "{text}");
}
