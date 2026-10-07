//! `CoordValues`: spacing detection, lookups and bounds.

use super::coord_values::{CoordValue, CoordValues, SpacingCheck};

fn numbers(values: &[f64], f32_source: bool) -> CoordValues {
    CoordValues::from_values(values.to_vec(), f32_source).expect("non-empty")
}

#[test]
fn evenly_spaced_values_are_stored_as_start_and_step() {
    let lat = numbers(&[90.0, 45.0, 0.0, -45.0, -90.0], false);
    assert_eq!(
        lat,
        CoordValues::Regular {
            start: 90.0,
            step: -45.0,
            len: 5
        }
    );
    assert_eq!(
        numbers(&[7.0], false),
        CoordValues::Regular {
            start: 7.0,
            step: 0.0,
            len: 1
        }
    );
    assert_eq!(CoordValues::from_values(Vec::new(), false), None);
}

#[test]
fn a_million_hourly_steps_cost_three_numbers() {
    let hours: Vec<f64> = (0..1_000_000).map(f64::from).collect();
    let time = numbers(&hours, false);
    assert!(matches!(time, CoordValues::Regular { len: 1_000_000, .. }));
    assert_eq!(time.number(876_543), Some(876_543.0));
}

#[test]
fn uneven_values_are_kept_and_read_exactly() {
    let plev = numbers(&[1000.0, 925.0, 850.0, 700.0, 500.0, 300.0], false);
    assert!(matches!(plev, CoordValues::Values(_)));
    assert_eq!(plev.get(2), Some(CoordValue::Number(850.0)));
    assert_eq!(plev.range_bounds(1, 3, 6), Some((700.0, 925.0)));
    // A NaN fill breaks the spacing instead of being smoothed over.
    assert!(matches!(
        numbers(&[0.0, f64::NAN, 2.0], false),
        CoordValues::Values(_)
    ));
}

#[test]
fn f32_rounding_counts_as_even_only_for_f32_sources() {
    // 0.1 degree longitudes as stored in f32, read back as f64.
    let lon: Vec<f64> = (0..3600)
        .map(|i| f64::from((-180.0 + 0.1 * f64::from(i)) as f32))
        .collect();
    assert!(matches!(
        numbers(&lon, true),
        CoordValues::Regular { len: 3600, .. }
    ));
    assert!(matches!(numbers(&lon, false), CoordValues::Values(_)));
}

#[test]
fn spacing_check_streams_one_value_at_a_time() {
    let check = SpacingCheck::new(0.0, 5, 40.0);
    assert!((0..5).all(|i| check.fits(i, 10.0 * i as f64)));
    assert!(!check.fits(3, 31.0));
    assert_eq!(check.expected(4), 40.0);
}

#[test]
fn endpoints_interpolate_and_mismatched_lengths_fall_back_to_them() {
    let ends = CoordValues::Endpoints {
        first: 0.0,
        last: 100.0,
        len: 11,
    };
    assert!(!ends.is_exact() && !ends.matches(11));
    assert_eq!(ends.number(5), Some(50.0));
    assert_eq!(ends.number(11), None);

    // Two stored values describing a 101-index dimension: interpolate over the dimension.
    let lat = numbers(&[-90.0, 90.0], false);
    assert_eq!(lat.number_for(50, 101), Some(0.0));
    assert_eq!(lat.range_bounds(25, 75, 101), Some((-45.0, 45.0)));
}

#[test]
fn labels_answer_by_index_and_have_no_numbers() {
    let regions = CoordValues::from_labels(vec!["Europe".into(), "Africa".into()]);
    let regions = regions.expect("labels");
    assert_eq!(regions.get(1), Some(CoordValue::Label("Africa")));
    assert_eq!(regions.label(0), Some("Europe"));
    assert_eq!(regions.number(0), None);
    assert_eq!(regions.range_bounds(0, 1, 2), None);
    assert_eq!(CoordValues::from_labels(Vec::new()), None);
}
