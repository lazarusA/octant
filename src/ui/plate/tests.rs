//! The plate browser's grid, fields and the plate kept across its fields.

use std::sync::Arc;

use super::cell_side;
use crate::app::OctantApp;
use crate::app::{PlateState, StoreKind};
use crate::data::DatasetMetadata;
use crate::data::ome::{OmeDataset, OmePlate};

#[test]
fn cells_fill_the_width_within_bounds() {
    assert_eq!(cell_side(18.0 + 12.0 * 20.0, 12), 20.0);
    assert_eq!(cell_side(1000.0, 12), 22.0);
    assert_eq!(cell_side(100.0, 48), 5.0);
}

#[test]
fn a_plate_contains_its_fields_only() {
    let plate = PlateState::new(
        "https://x/plate.zarr/",
        StoreKind::RemoteZarr,
        Arc::default(),
    );
    assert!(plate.contains("https://x/plate.zarr"));
    assert!(plate.contains("https://x/plate.zarr/B/2/1"));
    assert!(!plate.contains("https://x/plate.zarr2/B/2/1"));
    assert_eq!(plate.field_url("B/2", "1"), "https://x/plate.zarr/B/2/1");
}

fn plate_metadata() -> DatasetMetadata {
    DatasetMetadata {
        ome: Some(Arc::new(OmeDataset {
            plate: Some(Arc::new(OmePlate::default())),
            ..Default::default()
        })),
        ..Default::default()
    }
}

#[test]
fn the_plate_stays_while_its_fields_open_and_goes_after() {
    let mut app = OctantApp::default();
    app.layout.show_dataset_panel = false;
    app.selected.store_target = "https://x/plate.zarr".into();
    assert!(app.follow_plate(&plate_metadata()));
    assert!(app.layout.show_dataset_panel, "the browser shows");
    app.selected.store_target = "https://x/plate.zarr/A/1/0".into();
    assert!(!app.follow_plate(&DatasetMetadata::default()));
    assert!(app.plate.is_some(), "a field keeps the plate");
    app.selected.store_target = "https://y/other.zarr".into();
    app.follow_plate(&DatasetMetadata::default());
    assert!(app.plate.is_none(), "another dataset closes it");
}

#[test]
fn opened_fields_are_marked_on_arrival_and_the_plate_kept_when_reopened() {
    let wells = vec![crate::data::ome::plate::PlateWell {
        path: "A/1".into(),
        row: 0,
        column: 0,
    }];
    let meta = DatasetMetadata {
        ome: Some(Arc::new(OmeDataset {
            plate: Some(Arc::new(OmePlate {
                wells,
                ..Default::default()
            })),
            ..Default::default()
        })),
        ..Default::default()
    };
    let mut app = OctantApp::default();
    app.selected.store_target = "https://x/plate.zarr".into();
    app.follow_plate(&meta);
    app.selected.store_target = "https://x/plate.zarr/A/1/2".into();
    app.follow_plate(&DatasetMetadata::default());
    let plate = app.plate.as_ref().expect("plate");
    assert_eq!(plate.opened, Some((0, "2".to_string())));
    assert_eq!(plate.well, Some(0));
    app.selected.store_target = "https://x/plate.zarr".into();
    app.follow_plate(&meta);
    let plate = app.plate.as_ref().expect("plate");
    assert_eq!((plate.well, plate.opened.as_ref()), (Some(0), None));
}

#[test]
fn clearing_the_datasets_closes_the_plate() {
    let mut app = OctantApp::default();
    app.selected.store_target = "https://x/plate.zarr".into();
    app.follow_plate(&plate_metadata());
    assert!(app.plate.is_some());
    app.clear_all_datasets();
    assert!(app.plate.is_none());
}
