//! A typed source activates the open dataset it names, never one whose
//! address merely ends with it.

use crate::app::StoreKind;
use crate::app::test_support::{add_memory_dataset, memory_app};

#[test]
fn a_source_matches_whole_addresses_not_their_ends() {
    let (mut app, _) = memory_app();
    let meta = add_memory_dataset(&mut app, "memory://data/ba.zarr");
    let id = StoreKind::make_source_id(StoreKind::RemoteZarr, "memory://data/ba.zarr");
    if let Some(dataset) = app.dataset_manager.get_mut(&id) {
        dataset.metadata = Some(meta);
    }
    assert!(
        !app.try_activate_dataset("a.zarr"),
        "a.zarr is another store"
    );
    assert!(app.try_activate_dataset("memory://data/ba.zarr/"));
}
