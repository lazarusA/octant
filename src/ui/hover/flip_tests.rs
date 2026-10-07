//! Hover coordinates on blocks that orientation flipped: the screen row maps back to the
//! stored index before the coordinate is read.

use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use super::enrich::stored_offset;
use super::entries_2d::resolve_2d_plot_entries;
use crate::app::OctantApp;
use crate::data::backends::zarr::{GenericZarrBlockStore, fetch_block_with_progress};
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

/// A 5x4 `t2m(lat, lon)` grid with values `0..20` in storage order, plotted.
fn plotted_grid(store_url: &str, lat: &[f64]) -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    let f64_dt = DataType::from_metadata(&MetadataV3::new("float64")).expect("float64");
    for (name, values) in [("lat", lat), ("lon", &[0.0, 10.0, 20.0, 30.0][..])] {
        let len = values.len() as u64;
        let fill = FillValue::from(f64::NAN);
        let axis = ArrayBuilder::new(vec![len], vec![len], f64_dt.clone(), fill)
            .build(store.clone(), &format!("/{name}"))
            .expect("build axis");
        axis.store_metadata().expect("axis metadata");
        let all = ArraySubset::new_with_shape(vec![len]);
        axis.store_array_subset(&all, values).expect("axis values");
    }
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![5, 4], vec![5, 4], f32_dt, FillValue::from(0f32));
    builder.dimension_names(Some(["lat", "lon"]));
    let t2m = builder.build(store.clone(), "/t2m").expect("build t2m");
    t2m.store_metadata().expect("t2m metadata");
    let values: Vec<f32> = (0..20u8).map(f32::from).collect();
    let all = ArraySubset::new_with_shape(vec![5, 4]);
    t2m.store_array_subset(&all, &values).expect("t2m values");

    let meta = GenericZarrBlockStore::new(store.clone(), store_url, "zarr", "Zarr")
        .inspect()
        .expect("inspect");
    let idx = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "t2m");
    let request = SliceRequest::full_range("t2m", &[5, 4]);
    let block = fetch_block_with_progress(store, store_url, &request, None).expect("block");
    let mut app = OctantApp {
        plotted_dataset_metadata: Some(meta),
        plotted_variable_idx: idx.expect("t2m variable"),
        ..Default::default()
    };
    app.apply_2d_projection(&block, 1, 0, (0, 4), (0, 5), &[0, 0], true, 0);
    app
}

/// `(value, lat label)` hovered at the vertical screen position `ny` (0 = top).
fn hover_at(app: &OctantApp, ny: f32) -> (f32, String) {
    let meta = app.plotted_dataset_metadata.as_ref();
    let var = meta.and_then(|m| m.variables.get(app.plotted_variable_idx));
    let matrix = app.matrix_data.as_ref().expect("matrix");
    let (val, fields, _, _) = resolve_2d_plot_entries(app, matrix, meta, var, 0.1, ny, None);
    let lat = fields
        .iter()
        .find(|f| f.label == "lat")
        .map(|f| f.value.clone());
    (val, lat.unwrap_or_default())
}

#[test]
fn south_to_north_rows_hover_with_their_own_latitude() {
    let app = plotted_grid("flip_ascending", &[-60.0, -20.0, 0.0, 10.0, 50.0]);
    assert_eq!(app.plotted_flipped_dims, ["lat"]);
    // North renders at the top: the top row holds the last stored row (values 16..20).
    assert_eq!(hover_at(&app, 0.05), (16.0, "50.00°N".to_string()));
    assert_eq!(hover_at(&app, 0.95), (0.0, "60.00°S".to_string()));
}

#[test]
fn north_to_south_rows_are_not_remapped() {
    let app = plotted_grid("flip_descending", &[50.0, 10.0, 0.0, -20.0, -60.0]);
    assert!(app.plotted_flipped_dims.is_empty());
    assert_eq!(hover_at(&app, 0.05), (0.0, "50.00°N".to_string()));
    assert_eq!(hover_at(&app, 0.95), (16.0, "60.00°S".to_string()));
}

#[test]
fn stored_offset_reverses_only_flipped_dimensions() {
    let app = OctantApp {
        plotted_flipped_dims: vec!["lat".into()],
        ..Default::default()
    };
    assert_eq!(stored_offset(&app, "lat", 0, 5), 4);
    assert_eq!(stored_offset(&app, "lat", 4, 5), 0);
    assert_eq!(stored_offset(&app, "lon", 1, 4), 1);
    assert_eq!(stored_offset(&app, "lat", 9, 5), 0, "out of range clamps");
    assert_eq!(stored_offset(&app, "lat", 0, 0), 0);
}
