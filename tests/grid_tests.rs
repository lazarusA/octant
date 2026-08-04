use octant::utils::grid::{check_and_orient_axes, check_and_orient_axes_with_coords};

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

#[test]
fn test_grid_orientation_positive_up() {
    // positive="up" attribute triggers Y flip for ascending coordinates
    let raw = vec![1.0, 2.0, 3.0, 4.0];
    let dim_names = vec!["lat".to_string(), "lon".to_string()];
    let mut attrs = serde_json::Map::new();
    attrs.insert(
        "positive".to_string(),
        serde_json::Value::String("up".to_string()),
    );

    let (oriented, _, _) = check_and_orient_axes(raw, 2, 2, &dim_names, &attrs);
    assert_eq!(oriented, vec![3.0, 4.0, 1.0, 2.0]);
}

#[test]
fn test_grid_orientation_descending_longitude() {
    // longitude_orientation="descending" triggers X flip horizontally
    let raw = vec![1.0, 2.0, 3.0, 4.0];
    let dim_names = vec!["lat".to_string(), "lon".to_string()];
    let mut attrs = serde_json::Map::new();
    attrs.insert(
        "longitude_orientation".to_string(),
        serde_json::Value::String("descending".to_string()),
    );

    let (oriented, _, _) = check_and_orient_axes(raw, 2, 2, &dim_names, &attrs);
    assert_eq!(oriented, vec![2.0, 1.0, 4.0, 3.0]);
}

#[test]
fn test_grid_orientation_transpose_lon_lat() {
    // If dimension names are (lon, lat) / 3x2, transpose to (lat, lon) / 2x3
    let raw = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let dim_names = vec!["lon".to_string(), "lat".to_string()];
    let attrs = serde_json::Map::new();

    let (oriented, w, h) = check_and_orient_axes(raw, 3, 2, &dim_names, &attrs);
    assert_eq!(w, 2);
    assert_eq!(h, 3);
    assert_eq!(oriented.len(), 6);
}
