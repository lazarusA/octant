//! Colorbar labels, range reset, ticks and defaults.

use super::*;
use crate::data::{DatasetMetadata, VariableInfo, matrix_data::MatrixData};
use std::collections::HashMap;

#[test]
fn test_format_scientific_tick() {
    assert_eq!(format_scientific_tick(0.0), "0");
    assert_eq!(format_scientific_tick(42.0), "42");
    assert_eq!(format_scientific_tick(15.5), "15.5");
    assert_eq!(format_scientific_tick(100.25), "100.25");
    assert_eq!(format_scientific_tick(0.0000123), "1.23e-5");
    assert_eq!(format_scientific_tick(100000.0), "1e5");
    assert_eq!(format_scientific_tick(-50.0), "-50");
}

#[test]
fn test_colorbar_default_and_custom_label() {
    let mut app = OctantApp::default();
    assert_eq!(app.colorbar_label(), "Scalar Field");
    assert_eq!(app.default_colorbar_label(), "Scalar Field");

    app.layers.base.color.custom_label = Some("Surface Temp (Celsius)".to_string());
    assert_eq!(app.colorbar_label(), "Surface Temp (Celsius)");
    assert_eq!(app.default_colorbar_label(), "Scalar Field");

    app.reset_colorbar_label();
    assert_eq!(app.colorbar_label(), "Scalar Field");
    assert!(app.layers.base.color.custom_label.is_none());

    let mut attrs = HashMap::new();
    attrs.insert("units".to_string(), "degK".to_string());
    let var = VariableInfo {
        name: "air_temp".to_string(),
        data_type: "float32".to_string(),
        shape: vec![10, 10],
        dimension_names: vec!["y".to_string(), "x".to_string()],
        chunk_shape: vec![10, 10],
        file_size: 400,
        units: Some("degK".to_string()),
        long_name: None,
        time_coverage_start: None,
        time_coverage_end: None,
        temporal_resolution: None,
        attributes: attrs,
    };
    let meta = DatasetMetadata {
        name: "test_dataset".to_string(),
        store_type: "zarr".to_string(),
        variables: vec![var],
        dimension_coordinates: HashMap::new(),
    };
    app.layers.base.selection_mut().metadata = Some(meta);
    app.layers.base.selection_mut().variable_idx = 0;

    assert_eq!(app.default_colorbar_label(), "air_temp (degK)");
    assert_eq!(app.colorbar_label(), "air_temp (degK)");

    app.layers.base.color.custom_label = Some("Custom Temp".to_string());
    assert_eq!(app.colorbar_label(), "Custom Temp");
    assert_eq!(app.default_colorbar_label(), "air_temp (degK)");

    app.reset_colorbar_label();
    assert_eq!(app.colorbar_label(), "air_temp (degK)");
}

#[test]
fn test_colorbar_range_reset() {
    let mut app = OctantApp::default();

    let mdata = MatrixData::new(
        10,
        10,
        vec![12.0; 100],
        12.0,
        88.0,
        "test_ds".to_string(),
        1,
    );
    app.layers.base.data.matrix = Some(mdata);

    app.layers.base.color.range_min = 20.0;
    app.layers.base.color.range_max = 50.0;
    app.layers.base.color.lock_bounds = true;

    app.reset_color_range();

    assert_eq!(app.layers.base.color.range_min, 12.0);
    assert_eq!(app.layers.base.color.range_max, 88.0);
    assert!(!app.layers.base.color.lock_bounds);
}

#[test]
fn test_colorbar_ticks_generation_custom_bounds() {
    let ticks = generate_colorbar_ticks(10.0, 50.0, 0, 1.0);
    assert!(!ticks.is_empty());
    let major_ticks: Vec<_> = ticks.iter().filter(|t| t.is_major).collect();
    assert_eq!(major_ticks.len(), 5);

    assert!((major_ticks[0].val - 10.0).abs() < 1e-4);
    assert!((major_ticks[4].val - 50.0).abs() < 1e-4);
    assert!((major_ticks[2].val - 30.0).abs() < 1e-4);
}

#[test]
fn test_colorbar_transparency_default() {
    let app = OctantApp::default();
    assert_eq!(app.colorbar_transparency, 0.0);
}

#[test]
fn log_ticks_end_for_infinite_and_huge_ranges() {
    let ticks = generate_colorbar_ticks(1.0, f32::INFINITY, 1, 1.0);
    assert!(ticks.iter().any(|t| t.is_major));
    let wide = generate_colorbar_ticks(1e-30, 1e30, 1, 1.0);
    assert!(
        wide.iter().all(|t| t.is_major),
        "no minor ticks past 16 decades"
    );
    let narrow = generate_colorbar_ticks(1.0, 1e4, 1, 1.0);
    assert!(narrow.iter().any(|t| !t.is_major));
}
