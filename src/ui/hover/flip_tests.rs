//! Hover coordinates on blocks that orientation flipped: the screen row maps back to the
//! stored index before the coordinate is read.

use crate::app::VariableSelection;
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

    let zarr = GenericZarrBlockStore::new(store.clone(), store_url, "zarr", "Zarr");
    let mut meta = zarr.inspect().expect("inspect");
    let idx = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "t2m");
    // Plotting reads the variable's coordinates, which then join its metadata.
    let var = idx
        .and_then(|i| meta.variables.get(i))
        .expect("t2m variable");
    meta.dimension_coordinates = zarr.variable_coordinates(var).expect("coordinates");
    let request = SliceRequest::full_range("t2m", &[5, 4]);
    let block = fetch_block_with_progress(store, store_url, &request, None).expect("block");
    let mut app = OctantApp::default();
    *app.layers.base.selection_mut() = VariableSelection {
        metadata: Some(meta),
        variable_idx: idx.expect("t2m variable"),
        ..Default::default()
    };
    app.apply_2d_projection(&block, 1, 0, (0, 4), (0, 5), &[0, 0], true, 0);
    app
}

/// `(value, lat label)` hovered at the vertical screen position `ny` (0 = top).
fn hover_at(app: &OctantApp, ny: f32) -> (f32, String) {
    let meta = app.plotted().metadata.as_ref();
    let var = meta.and_then(|m| m.variables.get(app.plotted().variable_idx));
    let matrix = app.layers.base.data.matrix.as_ref().expect("matrix");
    let (val, fields, _, _) = resolve_2d_plot_entries(app, matrix, meta, var, 0.1, ny, None);
    let lat = fields
        .iter()
        .find(|f| &*f.label == "lat")
        .map(|f| f.value.clone());
    (val, lat.unwrap_or_default())
}

#[test]
fn south_to_north_rows_hover_with_their_own_latitude() {
    let app = plotted_grid("flip_ascending", &[-60.0, -20.0, 0.0, 10.0, 50.0]);
    assert_eq!(app.layers.base.data.flipped_dims, ["lat"]);
    // North renders at the top: the top row holds the last stored row (values 16..20).
    assert_eq!(hover_at(&app, 0.05), (16.0, "50.00°N".to_string()));
    assert_eq!(hover_at(&app, 0.95), (0.0, "60.00°S".to_string()));
}

#[test]
fn north_to_south_rows_are_not_remapped() {
    let app = plotted_grid("flip_descending", &[50.0, 10.0, 0.0, -20.0, -60.0]);
    assert!(app.layers.base.data.flipped_dims.is_empty());
    assert_eq!(hover_at(&app, 0.05), (0.0, "50.00°N".to_string()));
    assert_eq!(hover_at(&app, 0.95), (16.0, "60.00°S".to_string()));
}

#[test]
fn stored_offset_reverses_only_flipped_dimensions() {
    let mut app = OctantApp::default();
    app.layers.base.data.flipped_dims = vec!["lat".into()];
    assert_eq!(stored_offset(&app, "lat", 0, 5), 4);
    assert_eq!(stored_offset(&app, "lat", 4, 5), 0);
    assert_eq!(stored_offset(&app, "lon", 1, 4), 1);
    assert_eq!(stored_offset(&app, "lat", 9, 5), 0, "out of range clamps");
    assert_eq!(stored_offset(&app, "lat", 0, 0), 0);
}

/// A `(lat, lon)` block of stored lat rows `origin..origin + rows`, oriented north-up: its
/// row `k` holds stored row `origin + rows - 1 - k`, valued `stored_lat * 10 + lon`.
fn flipped_block(origin: usize, rows: usize, cols: usize) -> crate::data::OctantBlock {
    let values: Vec<f32> = (0..rows)
        .flat_map(|k| {
            let stored = origin + rows - 1 - k;
            (0..cols).map(move |c| (stored * 10 + c) as f32)
        })
        .collect();
    let dims = ["lat", "lon"].map(String::from).to_vec();
    let mut block = crate::data::OctantBlock::new(
        "t2m".into(),
        vec![rows, cols],
        dims,
        vec![origin, 0],
        values,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    block.flipped_dims = vec!["lat".into()];
    block
}

#[test]
fn a_view_inside_a_larger_flipped_block_shows_its_own_rows() {
    // Stored lat rows 2..5 out of a 10-row cached block, north first.
    let block = flipped_block(0, 10, 2);
    let rows = block.oriented_range(0, (2, 5));
    let mut app = OctantApp::default();
    app.apply_2d_projection(&block, 1, 0, (0, 2), rows, &[0, 0], true, 0);
    let matrix = app.layers.base.data.matrix.as_ref().expect("matrix");
    let first_column: Vec<f32> = matrix.values.iter().step_by(2).copied().collect();
    assert_eq!(first_column, [40.0, 30.0, 20.0]);
}

#[test]
fn a_volume_from_two_flipped_blocks_runs_north_to_south_throughout() {
    let mut app = OctantApp {
        selected: VariableSelection {
            plot_type: crate::plots::PlotType::Volume,
            ..Default::default()
        },
        ..Default::default()
    };
    // Both 5-row blocks feed one 10-row volume requested over stored rows 0..=9.
    for origin in [0, 5] {
        let block = flipped_block(origin, 5, 2);
        let (req, local) = (((0, 1), (0, 9), (0, 0)), ((0, 2), (0, 5), (0, 1)));
        app.apply_3d_volume_projection(
            &block,
            1,
            0,
            usize::MAX,
            req.0,
            req.1,
            req.2,
            local.0,
            local.1,
            local.2,
            &[0, 0],
            true,
            true,
            0,
        );
    }
    let volume = app.layers.base.data.volume.as_ref().expect("volume");
    assert_eq!((volume.width, volume.height), (2, 10));
    let first_column: Vec<f32> = volume.values.iter().step_by(2).copied().collect();
    let north_to_south: Vec<f32> = (0..10).rev().map(|lat| (lat * 10) as f32).collect();
    assert_eq!(first_column, north_to_south);
    // The 3D hover reverses the whole volume height, matching this layout.
    assert_eq!(stored_offset(&app, "lat", 0, 10), 9);
}
