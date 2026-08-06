use octant::utils::grid::check_and_orient_axes_with_coords;

#[test]
fn test_grid_orientation_esdc_example_a() {
    // Example A (ESDC): Axis data ascends [-89.875, ..., 89.875] (Row 0 = South [1.0, 2.0], Row 1 = North [3.0, 4.0])
    let raw = vec![1.0, 2.0, 3.0, 4.0];
    let dim_names = vec!["time".to_string(), "lat".to_string(), "lon".to_string()];
    let attrs = serde_json::Map::new();
    let lat_coords = vec![-89.875, 89.875];

    let (oriented, _, _) =
        check_and_orient_axes_with_coords(raw, 2, 2, &dim_names, &attrs, Some(&lat_coords), None);
    // Ascending lat axis SHOULD flip Y so North [3.0, 4.0] is at Row 0 (top of map)
    assert_eq!(oriented, vec![3.0, 4.0, 1.0, 2.0]);
}

#[test]
fn test_grid_orientation_seasfire_example_b() {
    // Example B (SeasFire): Axis data descends [89.875, ..., -89.875] (Row 0 = North [1.0, 2.0], Row 1 = South [3.0, 4.0])
    let raw = vec![1.0, 2.0, 3.0, 4.0];
    let dim_names = vec![
        "time".to_string(),
        "latitude".to_string(),
        "longitude".to_string(),
    ];
    let attrs = serde_json::Map::new();
    let lat_coords = vec![89.875, -89.875];

    let (oriented, _, _) = check_and_orient_axes_with_coords(
        raw.clone(),
        2,
        2,
        &dim_names,
        &attrs,
        Some(&lat_coords),
        None,
    );
    // Descending lat axis should NOT flip Y, Row 0 stays North [1.0, 2.0]
    assert_eq!(oriented, raw);
}
