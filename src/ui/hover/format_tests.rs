//! Tests of hover coordinate formatting.

use super::*;

fn ctx(name: &str, idx: usize, total_len: usize) -> DimContext<'_> {
    DimContext::resolve(None, None, None, name, idx, total_len)
}

fn numbers(values: &[f64]) -> CoordValues {
    CoordValues::from_values(values.to_vec(), false).expect("numbers")
}

fn labels(values: &[&str]) -> CoordValues {
    let labels = values.iter().map(|s| s.to_string()).collect();
    CoordValues::from_labels(labels).expect("labels")
}

#[test]
fn test_format_dimension_coord_fallback() {
    let res_dim = format_dimension_coord(None, None, None, "dim0", 5, 10, None);
    assert_eq!(res_dim, HoverField::new("dim0", "6 / 10"));

    let res_step = format_dimension_coord(None, None, None, "step", 5, 10, None);
    assert_eq!(res_step, HoverField::new("step", "Step: 6 / 10"));
}

#[test]
fn test_format_dimension_coord_geo_fallback() {
    let res_lon = format_dimension_coord(None, None, None, "lon", 0, 1, Some(-45.5));
    assert_eq!(res_lon, HoverField::new("lon", "45.50°W"));

    let res_lat = format_dimension_coord(None, None, None, "lat", 0, 1, Some(12.25));
    assert_eq!(res_lat, HoverField::new("lat", "12.25°N"));
}

#[test]
fn negative_coordinates_are_formatted_like_positive_ones() {
    let lons = numbers(&[-45.5, 45.5]);
    assert_eq!(
        from_coords(&lons, &ctx("lon", 0, 2)).as_deref(),
        Some("45.50°W")
    );
    assert_eq!(
        from_coords(&lons, &ctx("lon", 1, 2)).as_deref(),
        Some("45.50°E")
    );
}

#[test]
fn coordinates_show_their_grid_spacing() {
    // 0.001° cells: neighbors stay apart, the last one measured from its previous.
    let fine = numbers(&[45.121, 45.122, 45.123]);
    let at = |i| from_coords(&fine, &ctx("lat", i, 3));
    assert_eq!(at(0).as_deref(), Some("45.121°N"));
    assert_eq!(at(2).as_deref(), Some("45.123°N"));
    // Coarse grids keep two decimals.
    let coarse = numbers(&[10.0, 10.25]);
    assert_eq!(
        from_coords(&coarse, &ctx("lon", 1, 2)).as_deref(),
        Some("10.25°E")
    );
}

#[test]
fn centered_grids_show_their_offset() {
    // SeasFire: 0.25° cells centered from -179.875.
    let seasfire = CoordValues::Regular {
        start: -179.875,
        step: 0.25,
        len: 1440,
    };
    let at = |i| from_coords(&seasfire, &ctx("lat", i, 1440));
    assert_eq!(at(1019).as_deref(), Some("74.875°N"));
    assert_eq!(at(1020).as_deref(), Some("75.125°N"));
    // Centers stored as f32 keep three decimals, not their rounding noise.
    let stored = numbers(&[
        f64::from(74.885f32),
        f64::from(75.135f32),
        f64::from(75.385f32),
    ]);
    let at = |i| from_coords(&stored, &ctx("lat", i, 3));
    assert_eq!(at(0).as_deref(), Some("74.885°N"));
    assert_eq!(at(1).as_deref(), Some("75.135°N"));
    // ERA5: 0.25° from 0 keeps two.
    let era5 = CoordValues::Regular {
        start: 0.0,
        step: 0.25,
        len: 1440,
    };
    assert_eq!(
        from_coords(&era5, &ctx("lon", 1, 1440)).as_deref(),
        Some("0.25°E")
    );
}

#[test]
fn labels_and_dates_are_shown_as_stored() {
    let regions = labels(&["Europe", "Africa"]);
    assert_eq!(
        from_coords(&regions, &ctx("region", 1, 2)).as_deref(),
        Some("Africa")
    );
    let dates = labels(&[" 2024-01-15 ", "2024-01-16"]);
    assert_eq!(
        from_coords(&dates, &ctx("day", 0, 2)).as_deref(),
        Some("2024-01-15")
    );
}

#[test]
fn mismatched_label_list_is_not_read_by_index() {
    // Three labels for a ten-step dimension: none of them belongs to index 7.
    let members = labels(&["r1", "r2", "r3"]);
    assert_eq!(from_coords(&members, &ctx("member", 7, 10)), None);
    // A numeric range with the wrong length still interpolates its endpoints.
    let levels = numbers(&[0.0, 90.0]);
    let mid = from_coords(&levels, &ctx("depth", 5, 10));
    assert_eq!(mid.as_deref(), Some("50.00 m"));
}

#[test]
fn uneven_levels_show_their_stored_value() {
    let levels = numbers(&[1000.0, 925.0, 850.0, 700.0, 500.0, 300.0]);
    assert!(matches!(levels, CoordValues::Values(_)));
    let at = |i| from_coords(&levels, &ctx("depth", i, 6));
    assert_eq!(at(2).as_deref(), Some("850.00 m"));
    assert_eq!(at(5).as_deref(), Some("300.00 m"));
}
