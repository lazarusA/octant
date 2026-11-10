//! Unit tests for coordinate bounds and cache.

use crate::data::CoordValues;
use crate::data::backends::coord_bounds::get_cached_coord_values_scoped;
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

    let ends = |dim| {
        let c = coords_of(&store, "test_store", dim)?;
        Some((c.len(), c.first_number()?, c.last_number()?))
    };
    assert_eq!(ends("lat"), Some((3, -90.0, 90.0)));
    assert_eq!(ends("lon"), Some((3, 0.0, 360.0)));
}

/// The store's coordinate `dim` from any of its groups, through the cache.
fn coords_of(store: &Arc<MemoryStore>, url: &str, dim: &str) -> Option<CoordValues> {
    get_cached_coord_values_scoped(store.clone(), url, dim, None, &[], None)
}

#[test]
fn evicted_stores_read_their_coordinates_again() {
    use crate::data::backends::coord_bounds::evict_coord_values;
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
    let read = || coords_of(&store, " evict_store/", "lat").and_then(|c| c.last_number());
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
            None,
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
        get_cached_coord_values_scoped(store.clone(), "scoped_store", "lat", scope, &[], None)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(Some("a")), Some(2.0));
    assert_eq!(
        read(Some("/b/")),
        Some(7.0),
        "slashes trimmed, group b read"
    );
}

/// A coordinate whose chunk failed to read is read again by the next lookup (B5).
#[test]
fn failed_coordinate_reads_are_not_cached() {
    use crate::data::test_stores::FailingStore;
    use std::sync::atomic::Ordering;
    let inner = Arc::new(MemoryStore::new());
    axis(&inner, "/lat", &[1.0, 2.0]);
    let store = FailingStore::new(inner, |key, _| key.as_str().starts_with("lat/c"));
    let read = || {
        get_cached_coord_values_scoped(store.clone(), "failing_store", "lat", None, &[], None)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(), None, "the chunk fails");
    store.failing.store(false, Ordering::SeqCst);
    assert_eq!(read(), Some(2.0), "read again once the store answers");

    // A metadata read that times out is no missing coordinate either.
    let inner = Arc::new(MemoryStore::new());
    axis(&inner, "/lat", &[1.0, 2.0]);
    let store = FailingStore::new(inner, |key, _| key.as_str() == "lat/zarr.json");
    let read = || {
        get_cached_coord_values_scoped(store.clone(), "timing_out_store", "lat", None, &[], None)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(), None, "the metadata read times out");
    store.failing.store(false, Ordering::SeqCst);
    assert_eq!(read(), Some(2.0), "asked again once the store answers");
}

/// A coordinate of a type with no number reader is remembered as absent, not read again
/// for every block as a failed read is.
#[test]
fn unreadable_coordinate_types_are_cached_as_absent() {
    let store = Arc::new(MemoryStore::new());
    let bool_dt =
        DataType::from_metadata(&zarrs::metadata::v3::MetadataV3::new("bool")).expect("bool type");
    let lat = ArrayBuilder::new(vec![2], vec![2], bool_dt, FillValue::from(false))
        .build(store.clone(), "/lat")
        .expect("bool lat");
    lat.store_metadata().expect("metadata");
    lat.store_array_subset(&ArraySubset::new_with_shape(vec![2]), &[true, false])
        .expect("values");
    let read = || {
        get_cached_coord_values_scoped(store.clone(), "bool_lat_store", "lat", None, &[], None)
            .and_then(|c| c.last_number())
    };
    assert_eq!(read(), None, "bool is no coordinate");
    // Were it read again, the numeric array now at its path would answer.
    axis(&store, "/lat", &[1.0, 2.0]);
    assert_eq!(read(), None, "remembered as absent");
}

/// Without a scope, the coordinate cached for the first group in sorted order answers,
/// whatever order the cache holds them in (B5).
#[test]
fn unscoped_lookups_pick_one_group_every_time() {
    let store = Arc::new(MemoryStore::new());
    axis(&store, "/a/lat", &[1.0, 2.0]);
    axis(&store, "/b/lat", &[5.0, 6.0, 7.0]);
    for run in 0..50 {
        let url = format!("unscoped_store_{run}");
        let read = |scope: Option<&str>| {
            get_cached_coord_values_scoped(store.clone(), &url, "lat", scope, &[], None)
                .and_then(|c| c.last_number())
        };
        read(Some("b"));
        read(Some("a"));
        assert_eq!(read(None), Some(2.0), "run {run}");
    }
}
