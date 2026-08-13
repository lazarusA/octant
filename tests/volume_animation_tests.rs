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
    let vdata = block.volume(2, 1, 0, &fixed_indices, "test_volume").unwrap();

    assert_eq!(vdata.width, 4);  // X
    assert_eq!(vdata.height, 3); // Y
    assert_eq!(vdata.depth, 2);  // Z
    assert_eq!(vdata.values.len(), 24);
    assert_eq!(vdata.min_val, 0.0);
    assert_eq!(vdata.max_val, 23.0);
}
