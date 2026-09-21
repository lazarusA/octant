use super::{dataset::DatasetMetadata, tree::VariableTreeGroup, variable::VariableInfo};

#[test]
fn test_coord_bounds_for_range_with_boundary_coords() {
    let mut meta = DatasetMetadata::default();
    meta.dimension_coordinates.insert(
        "lat".to_string(),
        vec!["-90.0".to_string(), "90.0".to_string()],
    );

    // 101 points from index 0 (-90) to index 100 (+90)
    let bounds_full = meta.get_coord_bounds_for_range("lat", 101, (0, 100));
    assert_eq!(bounds_full, Some((-90.0, 90.0)));

    let bounds_sub = meta.get_coord_bounds_for_range("lat", 101, (25, 75));
    assert_eq!(bounds_sub, Some((-45.0, 45.0)));

    let bounds_single = meta.get_coord_bounds_for_range("lat", 101, (50, 50));
    assert_eq!(bounds_single, Some((0.0, 0.0)));
}

#[test]
fn test_coord_bounds_for_range_with_descending_boundary_coords() {
    let mut meta = DatasetMetadata::default();
    meta.dimension_coordinates.insert(
        "lat".to_string(),
        vec!["90.0".to_string(), "-90.0".to_string()],
    );

    // 101 points: index 0 is +90 (North), index 100 is -90 (South)
    // Range (0, 50) is Northern hemisphere [0..90]
    let bounds_north = meta.get_coord_bounds_for_range("lat", 101, (0, 50));
    assert_eq!(bounds_north, Some((0.0, 90.0)));

    // Range (50, 100) is Southern hemisphere [-90..0]
    let bounds_south = meta.get_coord_bounds_for_range("lat", 101, (50, 100));
    assert_eq!(bounds_south, Some((-90.0, 0.0)));
}

#[test]
fn test_coord_bounds_for_range_with_full_coords() {
    let mut meta = DatasetMetadata::default();
    meta.dimension_coordinates.insert(
        "lon".to_string(),
        vec![
            "0.0".to_string(),
            "10.0".to_string(),
            "25.0".to_string(),
            "50.0".to_string(),
        ],
    );

    let bounds_sub = meta.get_coord_bounds_for_range("lon", 4, (1, 3));
    assert_eq!(bounds_sub, Some((10.0, 50.0)));
}

#[test]
fn test_variable_tree_mixed_root_and_nested() {
    let vars = vec![
        VariableInfo {
            name: "elevation".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "mask".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "atmosphere/surface_pressure".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "atmosphere/forecast/u_wind".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "atmosphere/forecast/v_wind".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "ocean/temperature".to_string(),
            ..Default::default()
        },
    ];

    let tree = DatasetMetadata::build_tree_from_variables(&vars);

    assert_eq!(tree.variable_indices, vec![0, 1]); // elevation, mask
    assert_eq!(tree.subgroups.len(), 2); // atmosphere, ocean
    assert_eq!(tree.total_variable_count(), 6);

    let atmo = &tree.subgroups[0];
    assert_eq!(atmo.name, "atmosphere");
    assert_eq!(atmo.full_path, "atmosphere");
    assert_eq!(atmo.variable_indices, vec![2]); // surface_pressure
    assert_eq!(atmo.subgroups.len(), 1); // forecast

    let forecast = &atmo.subgroups[0];
    assert_eq!(forecast.name, "forecast");
    assert_eq!(forecast.full_path, "atmosphere/forecast");
    assert_eq!(forecast.variable_indices, vec![3, 4]); // u_wind, v_wind
    assert_eq!(forecast.total_variable_count(), 2);

    let ocean = &tree.subgroups[1];
    assert_eq!(ocean.name, "ocean");
    assert_eq!(ocean.full_path, "ocean");
    assert_eq!(ocean.variable_indices, vec![5]); // temperature
}

#[test]
fn test_variable_tree_filter() {
    let vars = vec![
        VariableInfo {
            name: "elevation".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "atmosphere/forecast/u_wind".to_string(),
            ..Default::default()
        },
        VariableInfo {
            name: "ocean/temperature".to_string(),
            ..Default::default()
        },
    ];

    let tree = VariableTreeGroup::build_tree_from_variables(&vars);

    // Search for "wind"
    let filtered_wind = tree.filter("wind", &vars).expect("should match wind");
    assert_eq!(filtered_wind.variable_indices.len(), 0);
    assert_eq!(filtered_wind.subgroups.len(), 1);
    assert_eq!(filtered_wind.subgroups[0].name, "atmosphere");
    assert_eq!(filtered_wind.subgroups[0].subgroups[0].name, "forecast");
    assert_eq!(
        filtered_wind.subgroups[0].subgroups[0].variable_indices,
        vec![1]
    );

    // Search for "ocean"
    let filtered_ocean = tree.filter("ocean", &vars).expect("should match ocean");
    assert_eq!(filtered_ocean.subgroups.len(), 1);
    assert_eq!(filtered_ocean.subgroups[0].name, "ocean");
    assert_eq!(filtered_ocean.subgroups[0].variable_indices, vec![2]);

    // Non-existent search
    assert!(tree.filter("nonexistent", &vars).is_none());
}
