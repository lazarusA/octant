//! Unit tests for tick mark formatting and axis utilities.

use super::ticks::TickMark;

#[test]
fn test_time_tick_formatting_cf_units() {
    let tick = TickMark::new_with_context(10.0, 1.0, 30.0, "time", Some("days since 2024-01-01"));
    assert_eq!(tick.as_str(), "01-11");
}

#[test]
fn test_time_tick_formatting_iso_units() {
    let tick = TickMark::new_with_context(15.0, 1.0, 30.0, "time", Some("2024-01-01"));
    assert_eq!(tick.as_str(), "01-16");
}

#[test]
fn test_time_tick_formatting_durations() {
    let tick_h = TickMark::new_with_context(24.0, 4.0, 48.0, "time", Some("hours"));
    assert_eq!(tick_h.as_str(), "24 h");

    let tick_d = TickMark::new_with_context(7.0, 1.0, 14.0, "time", Some("days"));
    assert_eq!(tick_d.as_str(), "7 d");
}

#[test]
fn test_time_tick_formatting_step() {
    let tick_step = TickMark::new_with_context(42.0, 1.0, 100.0, "step", None);
    assert_eq!(tick_step.as_str(), "t=42");
}

#[test]
fn test_standard_numeric_tick_formatting() {
    let tick_num = TickMark::new_with_context(123.45, 1.0, 100.0, "lon", None);
    assert_eq!(tick_num.as_str(), "123.45");
}

fn tick_values(start: f64, end: f64, title: &str, units: Option<&str>) -> Vec<f64> {
    super::steps::generate_ticks(start, end, title, units)
        .as_slice()
        .iter()
        .map(|t| t.val)
        .collect()
}

#[test]
fn degree_axes_tick_on_round_degrees() {
    // A 0.25° cell-centered global grid, shown edge to edge.
    let lon = tick_values(-179.875, 179.875, "lon [°E]", Some("degrees_east"));
    assert_eq!(lon, [-120.0, -60.0, 0.0, 60.0, 120.0]);
    let lat = tick_values(-90.125, 90.125, "latitude", Some("degrees_north"));
    assert_eq!(lat, [-90.0, -60.0, -30.0, 0.0, 30.0, 60.0, 90.0]);
}

#[test]
fn plain_axes_tick_on_one_two_five_steps() {
    assert_eq!(tick_values(40.3, 49.7, "y", None), [42.0, 44.0, 46.0, 48.0]);
    assert_eq!(
        tick_values(-0.5, 2047.5, "x", None),
        [0.0, 500.0, 1000.0, 1500.0, 2000.0]
    );
}

#[test]
fn ticks_sit_at_their_values() {
    // Descending axis (y growing downward): fract 0 at the start value.
    let ticks = super::steps::generate_ticks(100.0, 0.0, "y", None);
    for t in ticks.as_slice() {
        assert!((t.fract as f64 - (100.0 - t.val) / 100.0).abs() < 1e-6);
    }
    assert_eq!(ticks.as_slice().first().map(|t| t.as_str()), Some("0"));
}

#[test]
fn small_steps_label_with_their_decimals() {
    let ticks = super::steps::generate_ticks(0.0, 0.3, "x", None);
    let labels: Vec<&str> = ticks.as_slice().iter().map(|t| t.as_str()).collect();
    assert_eq!(
        labels,
        ["0.00", "0.05", "0.10", "0.15", "0.20", "0.25", "0.30"]
    );
}

#[test]
fn degenerate_axes_have_no_ticks() {
    assert!(
        super::steps::generate_ticks(5.0, 5.0, "x", None)
            .as_slice()
            .is_empty()
    );
    assert!(
        super::steps::generate_ticks(0.0, f64::NAN, "x", None)
            .as_slice()
            .is_empty()
    );
}
