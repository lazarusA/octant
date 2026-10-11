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
