//! Block coordinates: exact window values for uneven axes, exact ends for even ones, and
//! per-row coordinates kept in the order of the oriented data.

use std::collections::HashMap;
use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use super::block::{fetch_block_with_progress, window_coords};
use crate::data::CoordValues;
use crate::data::slice_request::SliceRequest;
use crate::utils::grid_flips::reverse_flipped_coordinates;

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
fn flipped_axes_reverse_only_their_per_row_coordinates() {
    let dims = ["lat", "lon"].map(String::from);
    let mut coords = HashMap::from([
        ("lat".to_string(), vec![-60.0, -20.0, 0.0, 10.0, 50.0]),
        ("lon".to_string(), vec![0.0, 30.0]),
    ]);
    reverse_flipped_coordinates(&mut coords, &dims, &[5, 4], (true, true));
    assert_eq!(coords["lat"], [50.0, 10.0, 0.0, -20.0, -60.0]);
    assert_eq!(
        coords["lon"],
        [0.0, 30.0],
        "an extent of a longer axis stays as is"
    );
}

#[test]
fn uneven_south_to_north_rows_get_coordinates_in_data_order() {
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
    axis("lat", &[-60.0, -20.0, 0.0, 10.0, 50.0]);
    axis("lon", &[0.0, 10.0, 20.0, 30.0]);

    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![5, 4], vec![5, 4], f32_dt, FillValue::from(f32::NAN));
    builder.dimension_names(Some(["lat", "lon"]));
    let t2m = builder.build(store.clone(), "/t2m").expect("build t2m");
    t2m.store_metadata().expect("t2m metadata");
    let values: Vec<f32> = (0..20u8).map(f32::from).collect();
    t2m.store_array_subset(&ArraySubset::new_with_shape(vec![5, 4]), &values)
        .expect("t2m values");

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
}
