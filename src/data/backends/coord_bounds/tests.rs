//! Unit tests for coordinate bounds and cache.

use crate::data::backends::coord_bounds::{
    fetch_all_dimension_coordinates, get_cached_coord_bounds,
};
use std::sync::Arc;
use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::storage::store::MemoryStore;

#[test]
fn test_fetch_and_cache_dimension_coordinates() {
    let store = Arc::new(MemoryStore::new());
    let meta = zarrs::metadata::v3::MetadataV3::new("float32");
    let float32_dt = DataType::from_metadata(&meta).unwrap();

    // Create a 1D lat array: [-90.0, 0.0, 90.0]
    let lat_builder = ArrayBuilder::new(
        vec![3],
        vec![3],
        float32_dt.clone(),
        FillValue::from(0.0f32),
    );
    let lat_array = lat_builder
        .build(store.clone(), "/lat")
        .expect("Failed building lat array");
    lat_array
        .store_metadata()
        .expect("Failed storing lat metadata");
    lat_array
        .store_array_subset(
            &ArraySubset::new_with_shape(vec![3]),
            &[-90.0f32, 0.0, 90.0],
        )
        .expect("Failed storing lat data");

    // Create a 1D lon array: [0.0, 180.0, 360.0]
    let lon_builder = ArrayBuilder::new(vec![3], vec![3], float32_dt, FillValue::from(0.0f32));
    let lon_array = lon_builder
        .build(store.clone(), "/lon")
        .expect("Failed building lon array");
    lon_array
        .store_metadata()
        .expect("Failed storing lon metadata");
    lon_array
        .store_array_subset(
            &ArraySubset::new_with_shape(vec![3]),
            &[0.0f32, 180.0, 360.0],
        )
        .expect("Failed storing lon data");

    let dim_names = vec!["lat".to_string(), "lon".to_string()];
    let coords = fetch_all_dimension_coordinates(store.clone(), &dim_names, Some("test_store"));

    let ends = |dim: &str| {
        let c = coords.get(dim)?;
        Some((c.len(), c.first_number()?, c.last_number()?))
    };
    assert_eq!(ends("lat"), Some((3, -90.0, 90.0)));
    assert_eq!(ends("lon"), Some((3, 0.0, 360.0)));

    // Test bounds
    let lat_bounds = get_cached_coord_bounds(store.clone(), "test_store", "lat");
    assert_eq!(lat_bounds, Some((-90.0, 90.0)));

    let lon_bounds = get_cached_coord_bounds(store.clone(), "test_store", "lon");
    assert_eq!(lon_bounds, Some((0.0, 360.0)));
}
