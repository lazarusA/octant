//! Blocks of arrays a `spatial:transform` places, with no coordinate arrays.

use std::sync::Arc;

use serde_json::json;
use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use super::generic::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

#[test]
fn blocks_take_their_coordinates_from_the_affine() {
    let store = Arc::new(MemoryStore::new());
    let dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![4, 8], vec![4, 8], dt, FillValue::from(0f32));
    builder.dimension_names(Some(["lat", "lon"]));
    let serde_json::Value::Object(attrs) = json!({
        "spatial:transform": [45.0, 0.0, -180.0, 0.0, -45.0, 90.0],
        "spatial:dimensions": ["lat", "lon"],
    }) else {
        unreachable!()
    };
    builder.attributes(attrs);
    let array = builder.build(store.clone(), "/g/rgb").expect("rgb");
    array.store_metadata().expect("rgb metadata");
    array
        .store_array_subset(&ArraySubset::new_with_shape(vec![4, 8]), &[1f32; 32])
        .expect("rgb values");

    // The group's grid keys never reach an array with its own affine.
    let group = json!({ "zarr_format": 3, "node_type": "group", "attributes": {
        "spatial:shape": [99, 99], "spatial:registration": "node" } });
    let key = zarrs::storage::StoreKey::new("g/zarr.json").expect("key");
    zarrs::storage::WritableStorageTraits::set(
        &*store,
        &key,
        group.to_string().into_bytes().into(),
    )
    .expect("group");
    let store = GenericZarrBlockStore::new(store, "memory://affine", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "g/rgb".into(),
        selections: vec![
            DimensionSelection::range(1, 3),
            DimensionSelection::range(2, 5),
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    // Even coordinates keep the window's ends.
    assert_eq!(block.coordinates.get("lat"), Some(&vec![22.5, -22.5]));
    assert_eq!(block.coordinates.get("lon"), Some(&vec![-67.5, 22.5]));
    assert!(block.flipped_dims.is_empty(), "already north-up");
}

#[test]
fn pyramid_levels_take_their_layout_entrys_affine() {
    use crate::data::ome::test_support::Fixture;
    // TESSERA's `global_rgb`: levels `0` and `1`, each its own affine, no lat/lon arrays.
    let level = |asset: &str, cell: f64| json!({ "asset": asset, "spatial:transform": [cell, 0.0, -180.0, 0.0, -cell, 90.0] });
    let mut coarse = level("1", 45.0);
    coarse["derived_from"] = json!("0");
    let layout = json!({
        "spatial:dimensions": ["lat", "lon"],
        "multiscales": { "layout": [level("0", 22.5), coarse] }
    });
    let store = Fixture::default()
        .v3_group("", json!({}))
        .v3_group("g", layout)
        .v3_group("g/0", json!({}))
        .v3_group("g/1", json!({}))
        .v3_array("g/1/rgb", &[4, 8], &["lat", "lon"])
        .memory();
    let store = GenericZarrBlockStore::new(store, "memory://pyramid", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "g/1/rgb".into(),
        selections: vec![
            DimensionSelection::range(0, 4),
            DimensionSelection::range(0, 8),
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(block.coordinates.get("lat"), Some(&vec![67.5, -67.5]));
    assert_eq!(block.coordinates.get("lon"), Some(&vec![-157.5, 157.5]));
}
