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
