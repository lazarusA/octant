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
    assert_eq!(max_allowed_large, 8); // strictly clamped to 8 steps on 128 MB limit!
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

#[test]
fn test_selected_volume_elements_and_limit() {
    use octant::data::DatasetMetadata;
    use octant::ui::variables_panel::{
        calculate_selected_volume_elements, is_volume_allowed_for_selection,
    };

    let mut app = OctantApp::default();
    let var_info = VariableInfo {
        name: "huge_var".to_string(),
        data_type: "f32".to_string(),
        shape: vec![50, 1000, 1000],
        chunk_shape: vec![1, 500, 500],
        dimension_names: vec!["z".to_string(), "y".to_string(), "x".to_string()],
        units: None,
        long_name: None,
        temporal_resolution: None,
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 50 * 1000 * 1000 * 4,
        attributes: HashMap::new(),
    };
    app.active_dataset_metadata = Some(DatasetMetadata {
        name: "test_ds".to_string(),
        store_type: "zarr".to_string(),
        variables: vec![var_info.clone()],
        dimension_coordinates: HashMap::new(),
    });
    app.selected_variable_idx = 0;
    app.selected_dim_ranges = vec![(0, 49), (0, 999), (0, 999)];
    app.dim_config = vec![
        octant::app::DimConfig {
            active: true,
            spatial: SpatialRole::Z,
            animation: AnimationRole::None,
        },
        octant::app::DimConfig {
            active: true,
            spatial: SpatialRole::Y,
            animation: AnimationRole::None,
        },
        octant::app::DimConfig {
            active: true,
            spatial: SpatialRole::X,
            animation: AnimationRole::None,
        },
    ];

    // 50 * 1000 * 1000 = 50,000,000 floats (200 MB) > 33,554,432 (128 MB)
    let elements = calculate_selected_volume_elements(&app);
    assert_eq!(elements, 50_000_000);
    assert!(!is_volume_allowed_for_selection(&app));

    // Reducing Z range to 10 slices: 10 * 1000 * 1000 = 10,000,000 floats (40 MB) <= 128 MB
    app.selected_dim_ranges[0] = (0, 9);
    let elements_small = calculate_selected_volume_elements(&app);
    assert_eq!(elements_small, 10_000_000);
    assert!(is_volume_allowed_for_selection(&app));
}

#[test]
fn test_prefetcher_abort_and_pending_bytes() {
    use octant::data::block_prefetch::BlockPrefetcher;

    let mut prefetcher = BlockPrefetcher::new();
    assert_eq!(prefetcher.pending_count(), 0);
    assert_eq!(prefetcher.pending_bytes(), 0);

    prefetcher.abort();
    assert_eq!(prefetcher.pending_count(), 0);
    assert_eq!(prefetcher.pending_bytes(), 0);
}

#[test]
fn test_known_truth_4d_analytical_voxel_evaluation() {
    use octant::data::{eval_known_truth_4d, generate_procedural_volume_4d, KnownTruth4DParams};

    let params = KnownTruth4DParams::default();
    let (nt, nz, ny, nx) = (5, 8, 12, 16);
    let (data, min_v, max_v) = generate_procedural_volume_4d(nt, nz, ny, nx);

    assert_eq!(data.len(), nt * nz * ny * nx);
    assert!(min_v >= params.background);
    assert!(max_v <= params.background + params.base_amplitude + params.amplitude_modulation);

    // Verify all voxels match analytical formula
    for t in 0..nt {
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let idx = t * (nz * ny * nx) + z * (ny * nx) + y * nx + x;
                    let actual = data[idx];
                    let expected = eval_known_truth_4d(t, nt, z, nz, y, ny, x, nx, Some(&params));
                    assert!(
                        (actual - expected).abs() < 1e-5,
                        "Voxel mismatch at t={t}, z={z}, y={y}, x={x}: actual={actual}, expected={expected}"
                    );
                }
            }
        }
    }
}

#[test]
fn test_known_truth_4d_peak_trajectory_tracking() {
    use octant::data::{
        generate_procedural_volume_4d, get_known_truth_4d_center, KnownTruth4DParams,
    };

    let (nt, nz, ny, nx) = (8, 32, 32, 32);
    let (data, _, _) = generate_procedural_volume_4d(nt, nz, ny, nx);
    let params = KnownTruth4DParams::default();

    for t in 0..nt {
        let (x0, y0, z0) = get_known_truth_4d_center(t, nt, Some(&params));

        // Find the voxel with the maximum value at timestep t
        let mut max_val = f32::NEG_INFINITY;
        let mut max_coords = (0, 0, 0);

        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let idx = t * (nz * ny * nx) + z * (ny * nx) + y * nx + x;
                    let val = data[idx];
                    if val > max_val {
                        max_val = val;
                        max_coords = (x, y, z);
                    }
                }
            }
        }

        // Convert discrete coords to normalized [0, 1]
        let norm_x = max_coords.0 as f32 / (nx - 1) as f32;
        let norm_y = max_coords.1 as f32 / (ny - 1) as f32;
        let norm_z = max_coords.2 as f32 / (nz - 1) as f32;

        let max_err = 1.0 / 31.0; // 1 voxel grid resolution
        assert!(
            (norm_x - x0).abs() <= max_err + 1e-4,
            "Peak X mismatch at t={t}: found={norm_x}, expected={x0}"
        );
        assert!(
            (norm_y - y0).abs() <= max_err + 1e-4,
            "Peak Y mismatch at t={t}: found={norm_y}, expected={y0}"
        );
        assert!(
            (norm_z - z0).abs() <= max_err + 1e-4,
            "Peak Z mismatch at t={t}: found={norm_z}, expected={z0}"
        );
    }
}

#[test]
fn test_4d_octant_block_volume_extraction_at_timesteps() {
    use octant::data::{
        generate_known_truth_4d_block, generate_procedural_volume_3d, VolumeData,
    };

    let (nt, nz, ny, nx) = (4, 6, 8, 10);
    let block = generate_known_truth_4d_block("test_4d_var", nt, nz, ny, nx);

    // Dim order: 0=time (T), 1=depth (Z), 2=lat (Y), 3=lon (X)
    // x_dim=3, y_dim=2, z_dim=1
    for t in 0..nt {
        let fixed_indices = vec![t, 0, 0, 0];
        let vdata = block
            .volume(3, 2, 1, &fixed_indices, "vol_4d_slice", true)
            .expect("Failed to extract volume from 4D block");

        assert_eq!(vdata.width, nx);
        assert_eq!(vdata.height, ny);
        assert_eq!(vdata.depth, nz);
        assert_eq!(vdata.values.len(), nx * ny * nz);

        // Compare against directly generated procedural 3D volume at timestep t
        let (expected_3d, exp_min, exp_max) = generate_procedural_volume_3d(nx, ny, nz, t, nt);
        assert_eq!(vdata.values, expected_3d);
        assert!((vdata.min_val - exp_min).abs() < 1e-5);
        assert!((vdata.max_val - exp_max).abs() < 1e-5);
    }

    // Also test VolumeData::new_procedural convenience constructor
    let proc_vol = VolumeData::new_procedural(nx, ny, nz, 1, nt);
    let (exp_vals_t1, _, _) = generate_procedural_volume_3d(nx, ny, nz, 1, nt);
    assert_eq!(proc_vol.values, exp_vals_t1);
}

#[test]
fn test_volume_animation_timeline_progression() {
    use octant::app::{AnimationRole, OctantApp, SpatialRole};
    use octant::data::generate_known_truth_4d_block;

    let mut app = OctantApp::default();
    let (nt, nz, ny, nx) = (6, 5, 8, 10);
    let block = generate_known_truth_4d_block("animated_salinity", nt, nz, ny, nx);

    // Setup dimension configs: dim 0 = Animated, dim 1 = Z, dim 2 = Y, dim 3 = X
    app.dim_config = vec![
        octant::app::DimConfig {
            spatial: SpatialRole::None,
            animation: AnimationRole::Animated,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::Z,
            animation: AnimationRole::None,
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
    app.animated_dim = Some(0);
    app.spatial_dims = vec![3, 2, 1]; // X, Y, Z
    app.selected_dim_indices = vec![0, 0, 0, 0];
    app.active_plot_type = octant::plots::PlotType::Volume;
    app.sync_plotted_state_from_selected();

    // Simulate stepping through all timesteps in OctantApp
    for step in 0..nt {
        app.current_timestep = step;
        app.selected_dim_indices[0] = step;
        app.sync_plotted_state_from_selected();
        app.apply_block_projection(&block);

        let vdata = app
            .volume_data
            .as_ref()
            .expect("VolumeData should be populated");
        assert_eq!(vdata.width, nx);
        assert_eq!(vdata.height, ny);
        assert_eq!(vdata.depth, nz);

        // Peak voxel at this step should match ground-truth trajectory
        let (x0, y0, z0) = octant::data::get_known_truth_4d_center(step, nt, None);
        let exp_x = (x0 * (nx - 1) as f32).round() as usize;
        let exp_y = (y0 * (ny - 1) as f32).round() as usize;
        let exp_z = (z0 * (nz - 1) as f32).round() as usize;

        let center_idx = exp_z * (nx * ny) + exp_y * nx + exp_x;
        let center_val = vdata.values[center_idx];
        assert!(
            center_val > 15.0,
            "Center voxel should have high amplitude, got {center_val} at step {step}"
        );

        // Entire volume slice should match the ground-truth 3D slice at timestep `step`
        let (expected_3d, exp_min, exp_max) =
            octant::data::generate_procedural_volume_3d(nx, ny, nz, step, nt);
        assert_eq!(vdata.values, expected_3d);
        assert!((vdata.min_val - exp_min).abs() < 1e-5);
        assert!((vdata.max_val - exp_max).abs() < 1e-5);
    }
}

#[test]
fn test_procedural_block_store_inspect_and_fetch() {
    use octant::data::{
        backends::ProceduralBlockStore, block_store::BlockStore, DimensionSelection, SliceRequest,
    };

    let store = ProceduralBlockStore::open("procedural://volume4d").unwrap();
    let meta = store.inspect().unwrap();
    assert_eq!(meta.variables.len(), 2);
    assert_eq!(meta.variables[0].name, "gaussian_wave_packet_4d");
    assert_eq!(meta.variables[0].shape, vec![20, 32, 32, 32]);
    assert_eq!(
        meta.variables[0].dimension_names,
        vec!["time", "depth", "lat", "lon"]
    );

    // Request timestep 3, full spatial
    let req = SliceRequest::new(
        "gaussian_wave_packet_4d",
        vec![
            DimensionSelection::Range { start: 3, end: 4 },
            DimensionSelection::Range { start: 0, end: 32 },
            DimensionSelection::Range { start: 0, end: 32 },
            DimensionSelection::Range { start: 0, end: 32 },
        ],
    );
    let block = store.fetch_block(&req).unwrap();
    assert_eq!(block.shape, vec![1, 32, 32, 32]);
    assert_eq!(block.origin, vec![3, 0, 0, 0]);
    assert_eq!(block.values.len(), 32 * 32 * 32);

    let vdata = block
        .volume(3, 2, 1, &[0, 0, 0, 0], "proc_test", true)
        .unwrap();
    assert_eq!(vdata.width, 32);
    assert_eq!(vdata.height, 32);
    assert_eq!(vdata.depth, 32);

    let (exp_3d, _, _) = octant::data::generate_procedural_volume_3d(32, 32, 32, 3, 20);
    assert_eq!(vdata.values, exp_3d);
}

#[test]
fn test_volume_dynamic_vs_locked_color_bounds() {
    use octant::app::{AnimationRole, OctantApp, SpatialRole};
    use octant::data::generate_known_truth_4d_block;

    let mut app = OctantApp::default();
    let (nt, nz, ny, nx) = (5, 6, 8, 10);
    let block = generate_known_truth_4d_block("pulse_4d", nt, nz, ny, nx);

    app.dim_config = vec![
        octant::app::DimConfig {
            spatial: SpatialRole::None,
            animation: AnimationRole::Animated,
            active: true,
        },
        octant::app::DimConfig {
            spatial: SpatialRole::Z,
            animation: AnimationRole::None,
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
    app.animated_dim = Some(0);
    app.spatial_dims = vec![3, 2, 1];
    app.selected_dim_indices = vec![0, 0, 0, 0];
    app.active_plot_type = octant::plots::PlotType::Volume;
    app.sync_plotted_state_from_selected();

    // 1. Dynamic Mode (!lock_color_bounds): color_range_min and max dynamically adapt to each 3D step
    app.lock_color_bounds = false;

    for step in 0..nt {
        app.current_timestep = step;
        app.plotted_selected_dim_indices[0] = step;
        app.apply_block_projection(&block);

        let vdata = app.volume_data.as_ref().unwrap();
        assert_eq!(app.color_range_min, vdata.min_val);
        assert_eq!(app.color_range_max, vdata.max_val);
        assert_eq!(app.volume_cmin, vdata.min_val);
        assert_eq!(app.volume_cmax, vdata.max_val);
    }

    // 2. Locked Mode (lock_color_bounds = true): bounds remain fixed
    app.color_range_min = 10.0;
    app.color_range_max = 80.0;
    app.volume_cmin = 10.0;
    app.volume_cmax = 80.0;
    app.lock_color_bounds = true;

    for step in 0..nt {
        app.current_timestep = step;
        app.plotted_selected_dim_indices[0] = step;
        app.apply_block_projection(&block);

        assert_eq!(app.color_range_min, 10.0);
        assert_eq!(app.color_range_max, 80.0);
        assert_eq!(app.volume_cmin, 10.0);
        assert_eq!(app.volume_cmax, 80.0);
    }

    // 3. Reset Bounds: unlocks and resets to active 3D volume min/max
    app.reset_color_range();
    assert!(!app.lock_color_bounds);
    let vdata = app.volume_data.as_ref().unwrap();
    assert_eq!(app.color_range_min, vdata.min_val);
    assert_eq!(app.color_range_max, vdata.max_val);
}


