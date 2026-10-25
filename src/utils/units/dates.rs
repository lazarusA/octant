//! The dates time coordinates stand for, as every reader of them (the
//! hover, the axes, voice commands) takes them.

use super::calendar::add_days_to_date;
use super::cf::{is_cf_time_unit, parse_iso_date, split_since, unit_to_milliseconds};
use crate::i18n::date::Date;

/// The date a time coordinate's value stands for, as every reader of dates
/// takes it: CF `"<unit> since <date>"` units, else (on a time dimension)
/// the days after a start date attribute (`time_coverage_start`), else a
/// Unix timestamp in seconds or milliseconds. `None` for other values, CF
/// durations among them.
pub fn coordinate_date(
    val: f64,
    units: Option<&str>,
    time_start: Option<&str>,
    is_time_dim: bool,
) -> Option<Date> {
    if let Some(u) = units
        && is_cf_time_unit(u)
    {
        return cf_since_date(val, u);
    }
    if is_time_dim
        && let Some(start_str) = time_start
        && let Some((y, m, d)) = parse_iso_date(start_str)
    {
        let (res_y, res_m, res_d) = add_days_to_date(y, m, d, val.round() as i64);
        return Some(Date::day(res_y, res_m, res_d));
    }
    // Unix timestamps (> 100M seconds or > 100B ms)
    if is_time_dim && val.abs() > 1e8 && val.abs() < 1e13 {
        let total_secs = if val.abs() > 1e11 {
            (val / 1000.0).round() as i64
        } else {
            val.round() as i64
        };
        return Some(date_after_hours(1970, 1, 1, total_secs.div_euclid(3600)));
    }
    None
}

/// `v` in CF units `"<unit> since <date>"` as a date at its hour.
pub(super) fn cf_since_date(v: f64, units: &str) -> Option<Date> {
    let (unit_part, ref_date_str) = split_since(units.trim())?;
    let (y, m, d) = parse_iso_date(ref_date_str).unwrap_or((1970, 1, 1));
    let scale_ms = unit_to_milliseconds(unit_part).unwrap_or(1_000) as f64;
    let total_hours = (v * scale_ms / 3_600_000.0).round() as i64;
    Some(date_after_hours(y, m, d, total_hours))
}

fn date_after_hours(y: usize, m: usize, d: usize, hours: i64) -> Date {
    let (res_y, res_m, res_d) = add_days_to_date(y, m, d, hours.div_euclid(24));
    Date::at_hour(res_y, res_m, res_d, hours.rem_euclid(24) as usize)
}
