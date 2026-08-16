use octant::app::{AnimationRole, OctantApp, SpatialRole};
use octant::data::{OctantBlock, VariableInfo, VolumeData};
use std::collections::HashMap;

#[test]
fn test_init_dimension_defaults_3d_dataset() {
    let mut app = OctantApp::default();
    let var_info = VariableInfo {
        name: "temperature".to_string(),
        data_type: "f32".to_string(),
        shape: vec![10, 20, 30],
        chunk_shape: vec![5, 10, 15],
        dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        units: Some("K".to_string()),
        long_name: Some("Surface Temperature".to_string()),
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 10 * 20 * 30 * 4,
        attributes: HashMap::new(),
    };

    octant::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var_info);

    // Lon should be X (dim 2)
    assert_eq!(app.dim_config[2].spatial, SpatialRole::X);
    // Lat should be Y (dim 1)
    assert_eq!(app.dim_config[1].spatial, SpatialRole::Y);
    // Time should be Z (dim 0) and Animated
    assert_eq!(app.dim_config[0].spatial, SpatialRole::Z);
    assert_eq!(app.dim_config[0].animation, AnimationRole::Animated);
    assert_eq!(app.animated_dim, Some(0));

    // spatial_dims should contain 3 dimensions in X, Y, Z order: [2, 1, 0]
    assert_eq!(app.spatial_dims, vec![2, 1, 0]);
}

#[test]
fn test_init_dimension_defaults_4d_dataset() {
    let mut app = OctantApp::default();
    let var_info = VariableInfo {
        name: "salinity".to_string(),
        data_type: "f32".to_string(),
        shape: vec![10, 5, 20, 30],
        chunk_shape: vec![5, 5, 10, 15],
        dimension_names: vec![
            "time".to_string(),
            "depth".to_string(),
            "lat".to_string(),
            "lon".to_string(),
        ],
        units: Some("PSU".to_string()),
        long_name: Some("Ocean Salinity".to_string()),
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 10 * 5 * 20 * 30 * 4,
        attributes: HashMap::new(),
    };

    octant::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var_info);

    // Lon -> X (dim 3), Lat -> Y (dim 2), Depth -> Z (dim 1), Time -> Animated (dim 0)
    assert_eq!(app.dim_config[3].spatial, SpatialRole::X);
    assert_eq!(app.dim_config[2].spatial, SpatialRole::Y);
    assert_eq!(app.dim_config[1].spatial, SpatialRole::Z);
    assert_eq!(app.dim_config[0].animation, AnimationRole::Animated);
    assert_eq!(app.animated_dim, Some(0));

    // spatial_dims in X, Y, Z order: [3, 2, 1]
    assert_eq!(app.spatial_dims, vec![3, 2, 1]);
}

#[test]
fn test_get_volume_shifts_for_spatial_dimensions() {
    let mut app = OctantApp::default();
    app.volume_data = Some(VolumeData::new(
        10,
        20,
        30,
        vec![0.0; 10 * 20 * 30],
        0.0,
        1.0,
        "test".to_string(),
    ));

    // Case 1: Z is animated (dim 0)
    app.plotted_dim_config = vec![
        octant::app::DimConfig {
            spatial: SpatialRole::Z,
            animation: AnimationRole::Animated,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::Y,
            animation: AnimationRole::None,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::X,
            animation: AnimationRole::None,
            active: true,
        },
    ];
    app.plotted_animated_dim = Some(0);

    app.current_timestep = 5;
    assert_eq!(app.get_volume_shifts(), (0, 0, 5));

    app.current_timestep = 35; // 35 % 30 = 5
    assert_eq!(app.get_volume_shifts(), (0, 0, 5));

    // Case 2: X is animated (dim 2)
    app.plotted_dim_config[0].animation = AnimationRole::None;
    app.plotted_dim_config[2].animation = AnimationRole::Animated;
    app.plotted_animated_dim = Some(2);

    app.current_timestep = 4;
    assert_eq!(app.get_volume_shifts(), (4, 0, 0));

    app.current_timestep = 14; // 14 % 10 = 4
    assert_eq!(app.get_volume_shifts(), (4, 0, 0));

    // Case 3: Y is animated (dim 1)
    app.plotted_dim_config[2].animation = AnimationRole::None;
    app.plotted_dim_config[1].animation = AnimationRole::Animated;
    app.plotted_animated_dim = Some(1);

    app.current_timestep = 7;
    assert_eq!(app.get_volume_shifts(), (0, 7, 0));

    app.current_timestep = 27; // 27 % 20 = 7
    assert_eq!(app.get_volume_shifts(), (0, 7, 0));
}

#[test]
fn test_octant_block_volume_extraction() {
    // 3D Block of shape [2 (Z), 3 (Y), 4 (X)]
    let shape = vec![2, 3, 4];
    let values: Vec<f32> = (0..24).map(|v| v as f32).collect();

    let block = OctantBlock::new(
        "var3d".to_string(),
        shape,
        vec!["z".into(), "y".into(), "x".into()],
        vec![0, 0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    );

    let fixed_indices = vec![0, 0, 0];
    let vdata = block
        .volume(2, 1, 0, &fixed_indices, "test_volume", true)
        .unwrap();

    assert_eq!(vdata.width, 4); // X
    assert_eq!(vdata.height, 3); // Y
    assert_eq!(vdata.depth, 2); // Z
    assert_eq!(vdata.values.len(), 24);
    assert_eq!(vdata.min_val, 0.0);
    assert_eq!(vdata.max_val, 23.0);
}

#[test]
fn test_line_profile_along_z_and_xyz() {
    let mut app = OctantApp::default();
    // 3D VolumeData with (width: 2, height: 3, depth: 4)
    // Total 24 floats with values 0.0..24.0
    // Index = z * 6 + y * 2 + x
    let values: Vec<f32> = (0..24).map(|v| v as f32).collect();
    app.volume_data = Some(VolumeData::new(
        2,
        3,
        4,
        values,
        0.0,
        23.0,
        "test_3d".to_string(),
    ));

    // 1. Along Z (dim 2) single profile at pixel 0 (x=0, y=0)
    app.line_profile_dim_idx = 2;
    app.line_profile_slice_idx = 0;
    app.line_plot_all_series = false;

    let (payload, len, count) = app.get_line_profile_payload();
    assert_eq!(len, 4);
    assert_eq!(count, 1);
    assert_eq!(payload, vec![0.0, 6.0, 12.0, 18.0]);

    // 2. Along Z single profile at pixel 1 (x=1, y=0)
    app.line_profile_slice_idx = 1;
    let (payload_p1, len_p1, count_p1) = app.get_line_profile_payload();
    assert_eq!(len_p1, 4);
    assert_eq!(count_p1, 1);
    assert_eq!(payload_p1, vec![1.0, 7.0, 13.0, 19.0]);

    // 3. Along X (dim 0) - extracted from matrix_data (timestep slice)
    app.line_profile_dim_idx = 0;
    app.line_profile_slice_idx = 0;
    app.matrix_data = Some(octant::data::MatrixData::new(
        2,
        3,
        vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
        10.0,
        60.0,
        "slice_t0".to_string(),
        1,
    ));

    let (payload_x, len_x, count_x) = app.get_line_profile_payload();
    assert_eq!(len_x, 2);
    assert_eq!(count_x, 1);
    assert_eq!(payload_x, vec![10.0, 20.0]);

    // Timestep advances -> new matrix_data slice
    app.matrix_data = Some(octant::data::MatrixData::new(
        2,
        3,
        vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0],
        100.0,
        600.0,
        "slice_t1".to_string(),
        1,
    ));
    let (payload_x1, len_x1, count_x1) = app.get_line_profile_payload();
    assert_eq!(len_x1, 2);
    assert_eq!(count_x1, 1);
    assert_eq!(payload_x1, vec![100.0, 200.0]);
}

#[test]
fn test_calculate_max_animated_steps_small_and_large_datasets() {
    // 1. Small dataset: 100 timesteps, 50x50 spatial -> 2500 f32s per step (10 KB/slice).
    // Total fits comfortably in 256 MB (67,108,864 elements).
    let var_small = VariableInfo {
        name: "small_var".to_string(),
        data_type: "f32".to_string(),
        shape: vec![100, 50, 50],
        chunk_shape: vec![10, 50, 50],
        dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        units: None,
        long_name: None,
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 100 * 50 * 50 * 4,
        attributes: HashMap::new(),
    };

    let dim_config = vec![
        octant::app::DimConfig {
            spatial: SpatialRole::None,
            animation: AnimationRole::Animated,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::Y,
            animation: AnimationRole::None,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::X,
            animation: AnimationRole::None,
            active: true,
        },
    ];
    let selected_ranges = vec![(0, 99), (0, 49), (0, 49)];

    let (max_allowed, requested, spatial_per_step) =
        octant::ui::variables_panel::calculate_max_animated_steps(
            &var_small,
            &dim_config,
            &selected_ranges,
            0,
        );
    assert_eq!(spatial_per_step, 2500);
    assert_eq!(requested, 100);
    assert_eq!(max_allowed, 100); // not clamped because 100 <= 67_108_864 / 2500 = 26843

    // 2. Large dataset: 500 timesteps, 2048x2048 spatial -> 4,194,304 f32s per step (16 MB/slice).
    // 67,108,864 / 4,194,304 = 16 steps maximum fit in 256 MB!
    let var_large = VariableInfo {
        name: "large_var".to_string(),
        data_type: "f32".to_string(),
        shape: vec![500, 2048, 2048],
        chunk_shape: vec![1, 1024, 1024],
        dimension_names: vec!["time".to_string(), "y".to_string(), "x".to_string()],
        units: None,
        long_name: None,
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 500 * 2048 * 2048 * 4,
        attributes: HashMap::new(),
    };
    let large_ranges = vec![(0, 499), (0, 2047), (0, 2047)];

    let (max_allowed_large, requested_large, spatial_large) =
        octant::ui::variables_panel::calculate_max_animated_steps(
            &var_large,
            &dim_config,
            &large_ranges,
            0,
        );
    assert_eq!(spatial_large, 2048 * 2048);
    assert_eq!(requested_large, 500);
    assert_eq!(max_allowed_large, 16); // strictly clamped to 16 steps!
}

#[test]
fn test_format_byte_size_and_calculate_download_sizes() {
    use octant::app::{AnimationRole, DimConfig, SpatialRole};
    use octant::ui::variables_panel::{calculate_download_sizes, format_byte_size};

    assert_eq!(format_byte_size(500), "500 B");
    assert_eq!(format_byte_size(1024), "1 KB");
    assert_eq!(format_byte_size(50 * 1024 * 1024), "50 MB");
    assert_eq!(format_byte_size(100 * 1024 * 1024 * 1024), "100 GB");
    assert_eq!(
        format_byte_size((1.5 * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64),
        "1.5 TB"
    );

    let var_info = VariableInfo {
        name: "test_var".to_string(),
        data_type: "f32".to_string(),
        shape: vec![100, 1000, 1000],
        chunk_shape: vec![1, 500, 500],
        dimension_names: vec!["time".to_string(), "y".to_string(), "x".to_string()],
        units: None,
        long_name: None,
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 100 * 1000 * 1000 * 4,
        attributes: HashMap::new(),
    };

    let dim_config = vec![
        DimConfig {
            active: true,
            spatial: SpatialRole::None,
            animation: AnimationRole::Animated,
        },
        DimConfig {
            active: true,
            spatial: SpatialRole::Y,
            animation: AnimationRole::None,
        },
        DimConfig {
            active: true,
            spatial: SpatialRole::X,
            animation: AnimationRole::None,
        },
    ];

    // Requesting 10 time steps out of 100 (full spatial)
    let selected_ranges = vec![(0, 9), (0, 999), (0, 999)];
    let (requested, total) = calculate_download_sizes(&var_info, &dim_config, &selected_ranges);

    assert_eq!(requested, 10 * 1000 * 1000 * 4); // 40 MB
    assert_eq!(total, 100 * 1000 * 1000 * 4); // 400 MB
}
