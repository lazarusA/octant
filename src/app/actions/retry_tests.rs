//! A command waiting for coordinates whose read fails: it runs once more
//! when the read settles and answers, never asking for them in a loop.

use std::sync::Arc;

use serde_json::json;
use zarrs::storage::store::MemoryStore;

use super::{Done, Failure};
use crate::app::test_support::{write_array, write_coordinate};
use crate::app::{OctantApp, StoreKind};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::{
    CoordValues, DataSource, Dataset, DatasetMetadata, SliceRequest, StoreHandle, VariableInfo,
};

const TARGET: &str = "memory://failing-coords";

/// A Zarr store whose coordinate reads fail, as an offline server's do.
struct FailingCoords(GenericZarrBlockStore);

impl BlockStore for FailingCoords {
    fn backend_name(&self) -> &str {
        "failing"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.0.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.0.inspect()
    }

    fn variable_coordinates(
        &self,
        _variable: &VariableInfo,
    ) -> Result<std::collections::HashMap<String, CoordValues>, BlockStoreError> {
        Err("HTTP 404 Not Found".into())
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.0.fetch_block_with_progress(request, on_progress)
    }
}

fn failing_app() -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    write_array(&store, "tas", &["time", "lat"], &[3, 2], &[1, 2]);
    let days = json!({ "units": "days since 2020-01-01" });
    write_coordinate(&store, "time", &[0.0, 31.0, 60.0], days);
    write_coordinate(&store, "lat", &[10.0, 20.0], json!({}));
    let zarr = GenericZarrBlockStore::new(store, TARGET, "zarr", "Zarr");
    let meta = zarr.inspect().expect("inspect");
    let source_id = StoreKind::make_source_id(StoreKind::RemoteZarr, TARGET);
    let kind = StoreKind::RemoteZarr.to_data_source_kind();
    let source = DataSource::new(&source_id, kind, TARGET, "Store");
    let handle = StoreHandle::new(source.clone(), Arc::new(FailingCoords(zarr)));
    let mut app = OctantApp::default();
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app
}

#[test]
fn a_failed_coordinate_read_is_asked_once() {
    let mut app = failing_app();
    let outcomes = super::dims_tests::run(&mut app, "select tas, then go to February 2020");
    // Without coordinates the date names no position: answered, not retried.
    let [Ok(Done::Selected(_)), Err(Failure::NoValue { .. })] = outcomes.as_slice() else {
        panic!("{outcomes:?}");
    };
    assert_eq!(app.coordinate_loader.pending_count(), 0, "asked again");
    assert!(app.resume_commands().is_empty(), "nothing left waiting");
}
