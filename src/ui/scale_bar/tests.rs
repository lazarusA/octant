//! Round scale bar lengths and the units they are written in.

use super::nice_length;

/// The length for `max_meters` is `meters`, written as `value unit`.
fn check(max_meters: f64, meters: f64, value: f64, unit: &str) {
    let (m, v, u) = nice_length(max_meters).expect("length");
    assert!((m - meters).abs() <= meters * 1e-12, "{m} != {meters}");
    assert_eq!((v, u), (value, unit));
}

#[test]
fn lengths_are_one_two_or_five_times_a_power_of_ten() {
    check(7.3e-6, 5e-6, 5.0, "µm");
    check(3.0e-6, 2e-6, 2.0, "µm");
    check(1.5e-6, 1e-6, 1.0, "µm");
    check(2.0e-6, 2e-6, 2.0, "µm");
}

#[test]
fn values_switch_units_at_a_thousand() {
    check(0.9e-3, 5e-4, 500.0, "µm");
    check(1.2e-3, 1e-3, 1.0, "mm");
    check(6.0e-8, 5e-8, 50.0, "nm");
    check(3.0e-13, 2e-13, 0.2, "pm");
}

#[test]
fn unusable_lengths_draw_nothing() {
    assert!(nice_length(0.0).is_none());
    assert!(nice_length(-1.0).is_none());
    assert!(nice_length(f64::NAN).is_none());
}
