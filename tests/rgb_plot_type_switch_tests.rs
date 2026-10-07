//! Tests for RGB composite plot type transitions across all supported plot types:
//! Heatmap, Sphere, Surface, Volume, PointCloud, Line.
//! Verifies that channel ranges, RGB composite mode, and TrueColor pixel
//! values are preserved across all transitions without falling back to grayscale.

use octant::app::{OctantApp, SpatialRole};
use octant::data::octant_block::OctantBlock;
use octant::plots::PlotType;
use std::sync::Arc;

/// Creates a synthetic 3-band RGB image block `[3, 8, 8]` with distinct R, G, B channels.
fn create_rgb_block() -> OctantBlock {
    let shape = vec![3, 8, 8];
    let strides = vec![64, 8, 1];
    let mut values = vec![0.0f32; 3 * 8 * 8];

    // Band 0: Red ramp (100.0 .. 255.0)
    for (i, v) in values[0..64].iter_mut().enumerate() {
        *v = 100.0 + (i as f32) * 2.0;
    }
    // Band 1: Green ramp (10.0 .. 50.0)
    for (i, v) in values[64..128].iter_mut().enumerate() {
        *v = 10.0 + (i as f32) * 0.5;
    }
    // Band 2: Blue constant (200.0)
    for v in &mut values[128..192] {
        *v = 200.0;
    }

    OctantBlock {
        values: Arc::from(values.into_boxed_slice()),
        origin: vec![0, 0, 0],
        shape,
        strides,
        dimension_names: vec!["band".to_string(), "y".to_string(), "x".to_string()],
        variable_name: "rgb_image".to_string(),
        min_value: 10.0,
        max_value: 255.0,
        coordinates: std::collections::HashMap::new(),
        attributes: std::collections::HashMap::new(),
        flipped_dims: Vec::new(),
    }
}

/// Sets up an `OctantApp` with an active 3-band RGB dataset.
fn setup_rgb_app() -> OctantApp {
    let mut app = OctantApp::default();
    let mut meta = octant::data::DatasetMetadata {
        name: "rgb_dataset".to_string(),
        ..Default::default()
    };
    meta.variables.push(octant::data::VariableInfo {
        name: "rgb_image".to_string(),
        data_type: "float32".to_string(),
        shape: vec![3, 8, 8],
        chunk_shape: vec![3, 8, 8],
        dimension_names: vec!["band".to_string(), "y".to_string(), "x".to_string()],
        units: None,
        long_name: Some("TrueColor RGB Image".to_string()),
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 3 * 8 * 8 * 4,
        attributes: std::collections::HashMap::new(),
    });

    app.load_new_metadata(meta);
    app.show_hero = false;
    app.rgb_composite_mode = true;
    app.rgb_composite_channels = [0, 1, 2];
    app
}

#[test]
fn test_2d_rgb_volume_slicing_produces_1depth_truecolor_volume() {
    let mut app = setup_rgb_app();
    app.active_plot_type = PlotType::Volume;
    let block = create_rgb_block();

    app.apply_block_projection(&block);

    assert!(app.volume_data.is_some(), "Volume data must be allocated");
    let vdata = app.volume_data.as_ref().unwrap();
    assert_eq!(vdata.width, 8);
    assert_eq!(vdata.height, 8);
    assert_eq!(
        vdata.depth, 1,
        "Planar 2D RGB image in Volume mode must have depth 1"
    );
    assert_eq!(vdata.values.len(), 64);

    // Verify packed TrueColor values (not all identical/grayscale)
    let first_voxel = vdata.values[0] as u32;
    let r = first_voxel & 0xFF;
    let g = (first_voxel >> 8) & 0xFF;
    let b = (first_voxel >> 16) & 0xFF;
    assert!(
        r > 0 || g > 0 || b > 0,
        "Voxel must have non-zero RGB components"
    );
}

#[test]
fn test_rgb_composite_transitions_across_all_plot_types() {
    let mut app = setup_rgb_app();
    let block = create_rgb_block();

    let plot_sequence = [
        PlotType::Heatmap,
        PlotType::Sphere,
        PlotType::Surface,
        PlotType::Volume,
        PlotType::PointCloud,
        PlotType::Heatmap,
        PlotType::Line,
        PlotType::Volume,
        PlotType::Heatmap,
    ];

    for &plot_type in &plot_sequence {
        app.switch_plot_type(plot_type);
        app.apply_block_projection(&block);

        // Verify channel dimension was NOT destroyed
        let ch_idx = app
            .channel_dim_index()
            .expect("Channel dimension must exist");
        assert_eq!(ch_idx, 0, "Band dimension must be Dim 0");
        assert_ne!(
            app.dim_config[ch_idx].spatial,
            SpatialRole::Z,
            "Channel dimension must never be converted to SpatialRole::Z in RGB mode"
        );
        assert_eq!(
            app.dim_config[ch_idx].range,
            (0, 2),
            "Channel range must cover all 3 bands (0..=2) after switching to {plot_type:?}"
        );
        assert!(
            app.rgb_composite_mode,
            "RGB composite mode must remain enabled across plot transitions"
        );

        // Verify data validity based on 2D vs 3D representation
        match plot_type {
            PlotType::Heatmap | PlotType::Sphere | PlotType::Surface | PlotType::Line => {
                assert!(
                    app.matrix_data.is_some(),
                    "Matrix data must be present for {plot_type:?}"
                );
                let mdata = app.matrix_data.as_ref().unwrap();
                let packed = mdata.values[0] as u32;
                let r = packed & 0xFF;
                let g = (packed >> 8) & 0xFF;
                let b = (packed >> 16) & 0xFF;
                assert!(
                    r != g || g != b,
                    "Pixels must remain TrueColor (R!=G or G!=B) in {plot_type:?}, got R={r}, G={g}, B={b}"
                );
            }
            PlotType::Volume | PlotType::PointCloud => {
                assert!(
                    app.volume_data.is_some(),
                    "Volume data must be present for {plot_type:?}"
                );
                let vdata = app.volume_data.as_ref().unwrap();
                assert_eq!(vdata.depth, 1);
                let packed = vdata.values[0] as u32;
                let r = packed & 0xFF;
                let g = (packed >> 8) & 0xFF;
                let b = (packed >> 16) & 0xFF;
                assert!(
                    r != g || g != b,
                    "Voxels must remain TrueColor in {plot_type:?}, got R={r}, G={g}, B={b}"
                );
            }
        }
    }
}
