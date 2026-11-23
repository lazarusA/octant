//! The time span a store states it holds data for, and the step of a time
//! axis inside it. ARCO-ERA5 allocates its whole time axis (from 1900 on) but
//! holds data only for the days its root attributes name; a missing chunk
//! reads as the fill value.

use super::calendar::days_from_civil;
use super::cf::{parse_iso_date, split_since, unit_to_milliseconds};
use crate::data::{CoordValues, VariableInfo};

/// The first and last days ARCO-ERA5's root attributes (inherited by every
/// variable) state data for: `valid_time_start`, and `valid_time_stop_era5t`
/// (its preliminary data, reaching furthest) or `valid_time_stop`. Only these:
/// ACDD `time_coverage_*` often lag stores that grow, whose newer steps hold
/// data.
pub fn stated_coverage(var: &VariableInfo) -> (Option<&str>, Option<&str>) {
    let attr = |k: &str| var.attributes.get(k).map(String::as_str);
    let end = attr("valid_time_stop_era5t").or_else(|| attr("valid_time_stop"));
    (attr("valid_time_start"), end)
}

/// The stated span in CF time `units`: its start, and its end, the last
/// moment of the end's day when it names no time of day (ARCO-ERA5 holds
/// every hour of `valid_time_stop_era5t`'s day).
pub fn stated_span(var: &VariableInfo, units: &str) -> (Option<f64>, Option<f64>) {
    let (start, end) = stated_coverage(var);
    let start = start.and_then(|d| date_in_units(d, units));
    let end = end.and_then(|d| {
        let at = date_in_units(d, units)?;
        if has_time_of_day(d) {
            return Some(at);
        }
        let day = 86_400_000.0 / unit_to_milliseconds(split_since(units.trim())?.0)? as f64;
        Some(at + day * (1.0 - 1e-9))
    });
    (start, end)
}

/// ISO `date` (its time of day too, if given) as a value in CF time `units`
/// (`"<unit> since <date>"`); `None` when either does not parse.
pub fn date_in_units(date: &str, units: &str) -> Option<f64> {
    let (unit, reference) = split_since(units.trim())?;
    let seconds = |s: &str| {
        let (y, m, d) = parse_iso_date(s)?;
        let days = days_from_civil(y as i64, m as u32, d as u32);
        Some(days as f64 * 86_400.0 + seconds_of_day(s)?)
    };
    let scale_ms = unit_to_milliseconds(unit)? as f64;
    Some((seconds(date)? - seconds(reference)?) * 1_000.0 / scale_ms)
}

/// The time of day of an ISO date-time, its digits, `:` and `.` only: what
/// follows (`Z`, ` UTC`, `+01:00`, `-05:00`) is a zone, ignored.
fn time_of_day(s: &str) -> &str {
    let Some((_, rest)) = s.trim().split_once(['T', ' ']) else {
        return "";
    };
    let rest = rest.trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '.'))
        .unwrap_or(rest.len());
    &rest[..end]
}

fn has_time_of_day(s: &str) -> bool {
    !time_of_day(s).is_empty()
}

/// The seconds after midnight of an ISO date-time (`2020-01-01T12:30:00Z`,
/// `1900-01-01 00:00:00 UTC`); 0 for a bare date, `None` for a time that does
/// not parse.
fn seconds_of_day(s: &str) -> Option<f64> {
    let mut parts = time_of_day(s).split(':').filter(|p| !p.is_empty());
    let mut field = || -> Option<f64> { parts.next().map_or(Some(0.0), |p| p.parse().ok()) };
    Some(field()? * 3_600.0 + field()? * 60.0 + field()?)
}

/// The step of the axis `coords` (`len` steps) to show instead of `current`
/// when `current` lies outside `[start, end]` (values in the axis' units):
/// the step of the range `lo..=hi` nearest `current` inside the span when
/// the range reaches into it (a range chosen partly over the data keeps
/// its covered part), else the last step at or before `end` (the newest
/// data, which a growing store like ARCO-ERA5 is mostly opened for, even
/// from a step before its start), or the first at or after `start`. `None` when `current` is inside, no step is, the axis
/// has no numbers, is only known by its endpoints (interpolated steps would
/// land off), or does not ascend (the bisection needs it).
pub fn step_inside(
    coords: &CoordValues,
    len: usize,
    current: usize,
    (lo, hi): (usize, usize),
    (start, end): (Option<f64>, Option<f64>),
) -> Option<usize> {
    let at = |i: usize| coords.number_for(i, len);
    if !coords.is_exact() || at(0)? > at(len.checked_sub(1)?)? {
        return None;
    }
    let value = at(current)?;
    if start.is_none_or(|s| value >= s) && end.is_none_or(|e| value <= e) {
        return None;
    }
    let first = match start {
        Some(s) => first_where(len, |i| Some(at(i)? >= s))?,
        None => 0,
    };
    let last = match end {
        Some(e) => first_where(len, |i| Some(at(i)? > e))?.checked_sub(1)?,
        None => len - 1,
    };
    if first > last {
        return None;
    }
    let (from, to) = (lo.max(first), hi.min(last));
    let step = if from <= to {
        current.clamp(from, to)
    } else if end.is_some() {
        last
    } else {
        first
    };
    (step != current).then_some(step)
}

/// The first of `0..len` where `pred` holds, `pred` false then true along
/// it (`len` when it never holds), by bisection: time axes run to millions
/// of steps.
fn first_where(len: usize, pred: impl Fn(usize) -> Option<bool>) -> Option<usize> {
    let (mut lo, mut hi) = (0, len);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid)? {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Some(lo)
}
