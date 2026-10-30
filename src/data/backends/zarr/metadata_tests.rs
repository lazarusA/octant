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

#[test]
fn array_opens_read_the_root_once() {
    let log = Arc::new(Mutex::new(Vec::<u8>::new()));
    let logged = UsageLogStorageAdapter::new(store(), log.clone(), String::new);
    let store = GenericZarrBlockStore::new(Arc::new(logged), "memory://groups", "zarr", "Zarr");
    let root_reads = || {
        let log = log.lock().unwrap_or_else(|p| p.into_inner());
        let text = String::from_utf8_lossy(&log);
        text.lines()
            .filter(|l| l.starts_with("get(zarr.json)"))
            .count()
    };
    let mut reads = Vec::new();
    for variable in ["a", "b", "c"] {
        let request = SliceRequest {
            variable: variable.into(),
            selections: vec![DimensionSelection::Index(0), DimensionSelection::Index(0)],
        };
        store.fetch_block(&request).expect("block");
        reads.push(root_reads());
    }
    // The first open reads the root; the others ask it nothing more.
    assert_eq!(reads, [reads[0]; 3]);
}
