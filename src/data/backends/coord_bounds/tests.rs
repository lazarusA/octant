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

#[test]
fn evicted_stores_read_their_coordinates_again() {
    use crate::data::backends::coord_bounds::{
        evict_coord_values, get_cached_coord_values_with_rank,
    };
    let store = Arc::new(MemoryStore::new());
    let dt =
        DataType::from_metadata(&zarrs::metadata::v3::MetadataV3::new("float64")).expect("float64");
    let lat = ArrayBuilder::new(vec![2], vec![2], dt, FillValue::from(f64::NAN))
        .build(store.clone(), "/lat")
        .expect("build lat");
    lat.store_metadata().expect("lat metadata");
    let all = ArraySubset::new_with_shape(vec![2]);
    lat.store_array_subset(&all, &[0.0f64, 1.0])
        .expect("lat values");
    let read = || {
        get_cached_coord_values_with_rank(store.clone(), " evict_store/", "lat", 0, 1)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(), Some(1.0));

    lat.store_array_subset(&all, &[0.0f64, 5.0])
        .expect("new lat values");
    assert_eq!(read(), Some(1.0), "served from the cache");
    evict_coord_values(Some("evict_store"));
    assert_eq!(read(), Some(5.0), "read again after eviction");
}

/// Writes a 1D float64 array of `values` at `path`.
fn axis(store: &Arc<MemoryStore>, path: &str, values: &[f64]) {
    let dt =
        DataType::from_metadata(&zarrs::metadata::v3::MetadataV3::new("float64")).expect("float64");
    let len = values.len() as u64;
    let a = ArrayBuilder::new(vec![len], vec![len], dt, FillValue::from(f64::NAN))
        .build(store.clone(), path)
        .expect("build axis");
    a.store_metadata().expect("axis metadata");
    a.store_array_subset(&ArraySubset::new_with_shape(vec![len]), values)
        .expect("axis values");
}

#[test]
fn grouped_variables_read_their_own_groups_coordinates() {
    use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
    use crate::data::{DatasetMetadata, VariableInfo};

    let store = Arc::new(MemoryStore::new());
    axis(&store, "/a/lat", &[-10.0, 0.0, 10.0]);
    axis(&store, "/b/lat", &[-40.0, -20.0, 0.0, 20.0, 40.0]);
    let var = |name: &str, len: u64| VariableInfo {
        name: name.into(),
        shape: vec![len],
        dimension_names: vec!["lat".into()],
        ..Default::default()
    };
    let variables = vec![var("a/temp", 3), var("b/temp", 5)];
    let meta = DatasetMetadata {
        dimension_coordinates: fetch_all_dimension_coordinates_for_variables(
            store,
            &variables,
            Some("grouped_store"),
        ),
        variables,
        ..Default::default()
    };
    let last = |var: &str| {
        meta.get_dim_coords(Some(var), "lat")
            .and_then(|c| c.last_number())
    };
    assert_eq!(last("a/temp"), Some(10.0));
    assert_eq!(last("b/temp"), Some(40.0), "not group a's latitude");
}

#[test]
fn scoped_lookups_read_their_group_before_any_cached_one() {
    use crate::data::backends::coord_bounds::get_cached_coord_values_scoped;

    let store = Arc::new(MemoryStore::new());
    axis(&store, "/a/lat", &[1.0, 2.0]);
    axis(&store, "/b/lat", &[5.0, 6.0, 7.0]);
    let read = |scope: Option<&str>| {
        get_cached_coord_values_scoped(store.clone(), "scoped_store", "lat", scope, &[], 0, 1)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(Some("a")), Some(2.0));
    assert_eq!(
        read(Some("/b/")),
        Some(7.0),
        "slashes trimmed, group b read"
    );
}
