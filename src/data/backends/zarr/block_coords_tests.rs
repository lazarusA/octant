//! Block coordinates: exact window values for uneven axes, exact ends for even ones, and
//! per-row coordinates kept in the order of the oriented data.

use std::collections::HashMap;
use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use super::block::fetch_block_with_progress;
use super::block_coords::window_coords;
use crate::data::CoordValues;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::grid_flips::{flipped_dims, reverse_flipped_coordinates};

fn numbers(values: &[f64]) -> CoordValues {
    CoordValues::from_values(values.to_vec(), false).expect("numbers")
}

#[test]
fn windows_of_uneven_axes_keep_every_value() {
    let plev = numbers(&[1000.0, 925.0, 850.0, 700.0, 500.0, 300.0]);
    assert_eq!(
        window_coords(&plev, 6, 1, 3),
        Some(vec![925.0, 850.0, 700.0])
    );
    let lon = numbers(&[0.0, 10.0, 20.0, 30.0, 40.0]);
    assert_eq!(
        window_coords(&lon, 5, 1, 3),
        Some(vec![10.0, 30.0]),
        "even: exact ends"
    );
    let ends = CoordValues::Endpoints {
        first: 0.0,
        last: 100.0,
        len: 11,
    };
    assert_eq!(window_coords(&ends, 11, 2, 4), Some(vec![20.0, 50.0]));
    assert_eq!(window_coords(&lon, 5, 4, 1), Some(vec![40.0, 40.0]));
}

#[test]
fn flipped_axes_reverse_their_coordinates_and_extents() {
    let dims = ["lat", "lon"].map(String::from);
    let mut coords = HashMap::from([
        ("lat".to_string(), vec![-60.0, -20.0, 0.0, 10.0, 50.0]),
        ("lon".to_string(), vec![0.0, 30.0]),
    ]);
    let flipped = flipped_dims(&dims, (true, true));
    assert_eq!(flipped, dims, "rows (lat) and columns (lon) both flip");
    reverse_flipped_coordinates(&mut coords, &dims, &[5, 4], &flipped);
    assert_eq!(coords["lat"], [50.0, 10.0, 0.0, -20.0, -60.0]);
    assert_eq!(coords["lon"], [30.0, 0.0], "an extent swaps its ends");
}

#[test]
fn a_partial_selection_of_a_flipped_extent_gets_its_own_coordinates() {
    // Regular ascending lat stored as its extent; the user selects stored rows 0..10.
    let dims = ["lat", "lon"].map(String::from);
    let shape = [181, 4];
    let mut coords = HashMap::from([("lat".to_string(), vec![-90.0, 90.0])]);
    reverse_flipped_coordinates(&mut coords, &dims, &shape, &["lat".to_string()]);
    let mut block = crate::data::OctantBlock::new(
        "v".into(),
        shape.to_vec(),
        dims.to_vec(),
        vec![0, 0],
        vec![0.0; 181 * 4],
        coords,
        HashMap::new(),
    );
    block.flipped_dims = vec!["lat".into()];
    let rows = block.oriented_range(0, (0, 10));
    let lat = crate::data::slicing::coords::extract_sliced_coords_for_dim(
        &block.coordinates,
        &block.dimension_names,
        &block.shape,
        0,
        rows,
    );
    assert_eq!(
        lat,
        Some(vec![-81.0, -90.0]),
        "north first, still in the south"
    );
}

/// A store with root `lat` and `lon` axes and a `t2m` array over dimensions `dims` whose
/// value at row `r`, column `c` is `r * lon.len() + c`.
fn store_with(lat: &[f64], lon: &[f64], dims: [&str; 2]) -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let f64_dt = DataType::from_metadata(&MetadataV3::new("float64")).expect("float64");
    let axis = |name: &str, values: &[f64]| {
        let len = values.len() as u64;
        let a = ArrayBuilder::new(
            vec![len],
            vec![len],
            f64_dt.clone(),
            FillValue::from(f64::NAN),
        )
        .build(store.clone(), &format!("/{name}"))
        .expect("build axis");
        a.store_metadata().expect("axis metadata");
        a.store_array_subset(&ArraySubset::new_with_shape(vec![len]), values)
            .expect("axis values");
    };
    axis("lat", lat);
    axis("lon", lon);

    let shape = vec![lat.len() as u64, lon.len() as u64];
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(
        shape.clone(),
        shape.clone(),
        f32_dt,
        FillValue::from(f32::NAN),
    );
    builder.dimension_names(Some(dims));
    let t2m = builder.build(store.clone(), "/t2m").expect("build t2m");
    t2m.store_metadata().expect("t2m metadata");
    let values: Vec<f32> = (0..lat.len() * lon.len()).map(|v| v as f32).collect();
    t2m.store_array_subset(&ArraySubset::new_with_shape(shape), &values)
        .expect("t2m values");
    store
}

#[test]
fn uneven_south_to_north_rows_get_coordinates_in_data_order() {
    let store = store_with(
        &[-60.0, -20.0, 0.0, 10.0, 50.0],
        &[0.0, 10.0, 20.0, 30.0],
        ["lat", "lon"],
    );
    let request = SliceRequest::full_range("t2m", &[5, 4]);
    let block = fetch_block_with_progress(store, "block_coords_store", &request, None)
        .expect("fetch block");
    // Rows flip so north is first; the per-row latitudes follow them.
    assert_eq!(block.coordinates["lat"], [50.0, 10.0, 0.0, -20.0, -60.0]);
    assert_eq!(block.coordinates["lon"], [0.0, 30.0]);
    assert_eq!(
        block.get(&[0, 0]),
        Some(16.0),
        "row 0 holds the northernmost data"
    );
    assert_eq!(
        block.flipped_dims,
        ["lat"],
        "only the rows run opposite to storage"
    );
}

#[test]
fn a_one_row_edge_block_flips_with_its_whole_latitude() {
    let store = store_with(
        &[-40.0, -20.0, 0.0, 20.0, 40.0],
        &[0.0, 10.0],
        ["lat", "lon"],
    );
    let request = SliceRequest::new(
        "t2m",
        vec![
            DimensionSelection::range(4, 5),
            DimensionSelection::range(0, 2),
        ],
    );
    let block =
        fetch_block_with_progress(store, "edge_block_store", &request, None).expect("fetch block");
    assert_eq!(
        block.flipped_dims,
        ["lat"],
        "decided from the whole latitude"
    );
    assert_eq!(block.shape, [1, 2]);
}

#[test]
fn generic_dimensions_flip_with_the_spatial_coordinates_they_hold() {
    let store = store_with(&[-10.0, 0.0, 10.0], &[0.0, 10.0], ["dim_0", "dim_1"]);
    let request = SliceRequest::full_range("t2m", &[3, 2]);
    let block = fetch_block_with_progress(store, "generic_dims_store", &request, None)
        .expect("fetch block");
    assert_eq!(block.flipped_dims, ["dim_0"]);
    assert_eq!(
        block.coordinates["dim_0"],
        [10.0, -10.0],
        "reversed with the rows"
    );
    assert_eq!(
        block.get(&[0, 0]),
        Some(4.0),
        "row 0 holds the northernmost data"
    );
}

#[test]
fn string_attributes_reach_blocks_without_json_quotes() {
    let store = Arc::new(MemoryStore::new());
    let dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![4, 1, 1], vec![4, 1, 1], dt, FillValue::from(0.0f32));
    builder.dimension_names(Some(["band", "y", "x"]));
    let mut attrs = serde_json::Map::new();
    attrs.insert("color_space".into(), "cmyk".into());
    builder.attributes(attrs);
    let ink = builder.build(store.clone(), "/ink").expect("build ink");
    ink.store_metadata().expect("ink metadata");
    let request = SliceRequest::full_range("ink", &[4, 1, 1]);
    let block = fetch_block_with_progress(store, "ink_store", &request, None).expect("fetch");
    assert_eq!(block.attributes["color_space"], "cmyk");
    assert!(crate::data::slicing::composite::cmyk::is_cmyk_block(
        &block, 0
    ));
}
