//! Climate and Forecast (CF) metadata and time unit parsing.

use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// Map of CF time unit names and short aliases to milliseconds per unit.
pub fn unit_to_milliseconds(unit: &str) -> Option<u64> {
    let clean = unit.trim();
    if clean.eq_ignore_ascii_case("millisecond")
        || clean.eq_ignore_ascii_case("milliseconds")
        || clean.eq_ignore_ascii_case("msec")
        || clean.eq_ignore_ascii_case("msecs")
        || clean.eq_ignore_ascii_case("ms")
    {
        Some(1)
    } else if clean.eq_ignore_ascii_case("second")
        || clean.eq_ignore_ascii_case("seconds")
        || clean.eq_ignore_ascii_case("sec")
        || clean.eq_ignore_ascii_case("secs")
        || clean.eq_ignore_ascii_case("s")
    {
        Some(1_000)
    } else if clean.eq_ignore_ascii_case("minute")
        || clean.eq_ignore_ascii_case("minutes")
        || clean.eq_ignore_ascii_case("min")
        || clean.eq_ignore_ascii_case("mins")
    {
        Some(60 * 1_000)
    } else if clean.eq_ignore_ascii_case("hour")
        || clean.eq_ignore_ascii_case("hours")
        || clean.eq_ignore_ascii_case("hr")
        || clean.eq_ignore_ascii_case("hrs")
        || clean.eq_ignore_ascii_case("h")
    {
        Some(60 * 60 * 1_000)
    } else if clean.eq_ignore_ascii_case("day")
        || clean.eq_ignore_ascii_case("days")
        || clean.eq_ignore_ascii_case("d")
    {
        Some(24 * 60 * 60 * 1_000)
    } else {
        None
    }
}

/// Checks if a unit string represents CF relative datetime or bare time durations without allocations.
#[inline]
pub fn is_cf_time_unit(unit: &str) -> bool {
    let clean = unit.trim();
    contains_ascii_case_insensitive(clean, "since")
        || contains_ascii_case_insensitive(clean, "hour")
        || contains_ascii_case_insensitive(clean, "day")
        || contains_ascii_case_insensitive(clean, "sec")
        || contains_ascii_case_insensitive(clean, "min")
        || contains_ascii_case_insensitive(clean, "year")
        || contains_ascii_case_insensitive(clean, "month")
        || clean.eq_ignore_ascii_case("h")
        || clean.eq_ignore_ascii_case("d")
        || clean.eq_ignore_ascii_case("s")
        || clean.eq_ignore_ascii_case("hr")
        || clean.eq_ignore_ascii_case("hrs")
        || clean.eq_ignore_ascii_case("ms")
}

/// Parses CF time unit string (e.g. "seconds since 1970-01-01" or bare duration units "hours").
/// Returns `(scale_in_ms, offset_timestamp_ms)`.
pub fn parse_time_unit(units_str: Option<&str>) -> (u64, i64) {
    let units = match units_str {
        Some(u) if !u.trim().is_empty() && !u.eq_ignore_ascii_case("default") => u.trim(),
        _ => return (1, 0),
    };

    if let Some((unit_part, _ref_date_str)) = split_since(units) {
        let scale = unit_to_milliseconds(unit_part).unwrap_or(1);
        (scale, 0)
    } else if let Some(scale) = unit_to_milliseconds(units) {
        (scale, 0)
    } else {
        (1, 0)
    }
}

/// Helper to split `"unit since date"` case-insensitively with zero allocation.
pub fn split_since(units: &str) -> Option<(&str, &str)> {
    let bytes = units.as_bytes();
    for i in 0..bytes.len().saturating_sub(6) {
        if bytes[i..i + 7].eq_ignore_ascii_case(b" since ") {
            return Some((units[..i].trim(), units[i + 7..].trim()));
        }
    }
    None
}

/// Parses ISO date strings like "2024-01-01" or "2024-01-01T00:00:00" into `(year, month, day)`.
pub fn parse_iso_date(s: &str) -> Option<(usize, usize, usize)> {
    let clean = s.trim();
    let date_part = clean.split(['T', ' ']).next().unwrap_or(clean);
    let mut parts = date_part.split('-');
    let y = parts.next()?.parse().ok()?;
    let m = parts.next()?.parse().ok()?;
    let d = parts.next()?.parse().ok()?;
    Some((y, m, d))
}

/// Dynamically parse reference date from CF unit string (e.g. "days since 1970-01-01", "hours since 2000-01-01")
/// or fallback to dataset target path hints if available.
pub fn parse_reference_date(
    units_str: Option<&str>,
    time_start: Option<&str>,
    temp_res: Option<&str>,
    target_hint: Option<&str>,
) -> (usize, usize, usize, usize) {
    // 1. Try explicit time_coverage_start (e.g. "1979-01-01T00:00:00" or "2001-01-01")
    let ref_date = time_start.and_then(parse_iso_date);

    // 2. Try temporal_resolution (e.g. "8D" -> 8, "16D" -> 16, "1D" -> 1)
    let days_step = if let Some(res) = temp_res {
        let clean = res.trim();
        if clean.ends_with('D') || clean.ends_with('d') {
            clean[..clean.len() - 1].parse::<usize>().unwrap_or(8)
        } else if contains_ascii_case_insensitive(clean, "day") {
            clean
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1)
        } else {
            8
        }
    } else if let Some(target) = target_hint {
        if contains_ascii_case_insensitive(target, "16d") {
            16
        } else if contains_ascii_case_insensitive(target, "8d")
            || contains_ascii_case_insensitive(target, "seasfire")
        {
            8
        } else if contains_ascii_case_insensitive(target, "1d")
            || contains_ascii_case_insensitive(target, "daily")
        {
            1
        } else {
            8
        }
    } else {
        8
    };

    if let Some((y, m, d)) = ref_date {
        return (y, m, d, days_step);
    }

    // 3. Fallback to CF units_str (e.g. "days since 1970-01-01")
    if let Some(u) = units_str
        && let Some((unit_part, ref_part)) = split_since(u)
        && let Some((y, m, d)) = parse_iso_date(ref_part)
    {
        let step = if unit_part.eq_ignore_ascii_case("day")
            || unit_part.eq_ignore_ascii_case("days")
            || unit_part.eq_ignore_ascii_case("d")
        {
            1
        } else {
            days_step
        };
        return (y, m, d, step);
    }

    // 4. Target hint check for dataset specific reference dates
    if let Some(target) = target_hint
        && contains_ascii_case_insensitive(target, "seasfire")
    {
        return (2001, 1, 1, days_step);
    }

    // Default reference date (1979-01-01 for ERA5 / ESDC)
    (1979, 1, 1, days_step)
}
