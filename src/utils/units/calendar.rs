//! Proleptic Gregorian civil calendar calculations in O(1) time.

/// Converts civil year, month (1..=12), and day (1..=31) to days since 1970-01-01 (epoch 0).
/// Negative values represent days before 1970-01-01.
#[inline]
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32; // [0, 399]
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + (doe as i64) - 719468
}

/// Converts days since 1970-01-01 (epoch 0) to civil (year, month 1..=12, day 1..=31).
#[inline]
pub fn civil_from_days(z: i64) -> (usize, usize, usize) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y.max(0) as usize, m as usize, d as usize)
}

/// Dynamically add N days (positive or negative) to a starting date (year, month, day) in O(1) time.
#[inline]
pub fn add_days_to_date(
    start_year: usize,
    start_month: usize,
    start_day: usize,
    days_to_add: i64,
) -> (usize, usize, usize) {
    let base = days_from_civil(
        start_year as i64,
        start_month.clamp(1, 12) as u32,
        start_day.clamp(1, 31) as u32,
    );
    let target = base.saturating_add(days_to_add);
    civil_from_days(target)
}
