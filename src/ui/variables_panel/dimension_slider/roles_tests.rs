//! Role changes and the ranges they leave: a time axis that is both Z and
//! animated keeps its one-chunk default range across frames, and a dimension
//! made spatial spans one chunk, never the whole axis.

use std::collections::HashMap;

use super::roles::apply_role_change;
use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::{DatasetMetadata, VariableInfo};

/// An app with an ERA5-like `t2m(valid_time, latitude, longitude)` selected
/// and its default roles, `time_chunk` steps per time chunk.
fn era5_like(time_chunk: u64) -> OctantApp {
    let var = VariableInfo {
        name: "t2m".to_string(),
        data_type: "float32".to_string(),
        shape: vec![1000, 721, 1440],
        dimension_names: vec!["valid_time".into(), "latitude".into(), "longitude".into()],
        chunk_shape: vec![time_chunk, 721, 1440],
        file_size: 0,
        units: None,
        long_name: None,
        time_coverage_start: None,
        time_coverage_end: None,
        temporal_resolution: None,
        attributes: HashMap::new(),
        ome: None,
    };
    let mut app = OctantApp::default();
    app.selected.metadata = Some(DatasetMetadata {
        name: "era5".to_string(),
        store_type: "Zarr".to_string(),
        variables: vec![var.clone()],
        dimension_coordinates: HashMap::new(),
        ome: None,
    });
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    app
}

/// What the slider row does every frame: applies each dimension's roles.
fn frame(app: &mut OctantApp) {
    for i in 0..app.selected.dim_config.len() {
        let c = app.selected.dim_config[i];
        apply_role_change(i, c.spatial, c.animation, app);
    }
}

#[test]
fn an_animated_z_time_axis_keeps_its_chunk_range_across_frames() {
    let mut app = era5_like(1);
    let time = app.selected.dim_config[0];
    assert_eq!(
        (time.spatial, time.animation),
        (SpatialRole::Z, AnimationRole::Animated)
    );
    assert_eq!(app.selected.dim_ranges[0], (0, 0));
    for _ in 0..3 {
        frame(&mut app);
    }
    assert_eq!(app.selected.dim_ranges[0], (0, 0), "time stays one chunk");
    assert_eq!(app.selected.dim_ranges[1], (0, 720));
}

#[test]
fn a_dimension_made_spatial_spans_the_chunk_holding_its_index() {
    let mut app = era5_like(24);
    assert_eq!(
        app.selected.dim_ranges[0],
        (0, 23),
        "the default is one chunk"
    );
    apply_role_change(0, SpatialRole::None, AnimationRole::Animated, &mut app);
    app.selected.dim_ranges[0] = (30, 30);
    apply_role_change(0, SpatialRole::Z, AnimationRole::Animated, &mut app);
    assert_eq!(app.selected.dim_ranges[0], (24, 47));
}

#[test]
fn the_last_chunk_ends_at_the_dimension() {
    let mut app = era5_like(24);
    apply_role_change(0, SpatialRole::None, AnimationRole::Animated, &mut app);
    app.selected.dim_ranges[0] = (995, 995);
    apply_role_change(0, SpatialRole::Z, AnimationRole::Animated, &mut app);
    assert_eq!(app.selected.dim_ranges[0], (984, 999));
}
