/// Calculate uncompressed file/variable size in bytes based on shape and data type string.
pub fn calculate_variable_size_bytes(shape: &[u64], data_type: &str) -> u64 {
    let element_count: u64 = shape.iter().product();
    let bytes_per_elem: u64 = match data_type.to_lowercase().as_str() {
        "float64" | "double" | "f64" | "i64" | "u64" | "int64" | "uint64" => 8,
        "float32" | "float" | "f32" | "i32" | "u32" | "int32" | "uint32" => 4,
        "float16" | "f16" | "i16" | "u16" | "int16" | "uint16" => 2,
        "i8" | "u8" | "int8" | "uint8" | "bool" => 1,
        _ => 4,
    };
    element_count.saturating_mul(bytes_per_elem)
}

/// Map of CF time unit names and short aliases to milliseconds per unit.
pub fn unit_to_milliseconds(unit: &str) -> Option<u64> {
    let clean_unit = unit.trim().to_lowercase();
    match clean_unit.as_str() {
        "millisecond" | "milliseconds" | "msec" | "msecs" | "ms" => Some(1),
        "second" | "seconds" | "sec" | "secs" | "s" => Some(1_000),
        "minute" | "minutes" | "min" | "mins" => Some(60 * 1_000),
        "hour" | "hours" | "hr" | "hrs" | "h" => Some(60 * 60 * 1_000),
        "day" | "days" | "d" => Some(24 * 60 * 60 * 1_000),
        _ => None,
    }
}

/// Parses CF time unit string (e.g. "seconds since 1970-01-01" or bare duration units "hours").
/// Returns `(scale_in_ms, offset_timestamp_ms)`.
pub fn parse_time_unit(units_str: Option<&str>) -> (u64, i64) {
    let units = match units_str {
        Some(u) if !u.trim().is_empty() && u != "Default" => u.trim(),
        _ => return (1, 0),
    };

    let lower = units.to_lowercase();
    if let Some((unit_part, _ref_date_str)) = lower.split_once(" since ") {
        let scale = unit_to_milliseconds(unit_part.trim()).unwrap_or(1);
        (scale, 0)
    } else if let Some(scale) = unit_to_milliseconds(&lower) {
        (scale, 0)
    } else {
        (1, 0)
    }
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
        let clean = res.trim().to_uppercase();
        if clean.ends_with('D') {
            clean.trim_end_matches('D').parse::<usize>().unwrap_or(8)
        } else if clean.ends_with("DAY") || clean.ends_with("DAYS") {
            clean
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1)
        } else {
            8
        }
    } else if let Some(target) = target_hint {
        let target_lower = target.to_lowercase();
        if target_lower.contains("16d") {
            16
        } else if target_lower.contains("8d") || target_lower.contains("seasfire") {
            8
        } else if target_lower.contains("1d") || target_lower.contains("daily") {
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
    if let Some(u) = units_str {
        let lower = u.to_lowercase();
        if let Some((_, ref_part)) = lower.split_once(" since ") {
            if let Some((y, m, d)) = parse_iso_date(ref_part.trim()) {
                let step = if lower.starts_with("day") || lower.starts_with("d ") {
                    1
                } else {
                    days_step
                };
                return (y, m, d, step);
            }
        }
    }

    // 4. Target hint check for dataset specific reference dates
    if let Some(target) = target_hint {
        let target_lower = target.to_lowercase();
        if target_lower.contains("seasfire") {
            return (2001, 1, 1, days_step);
        }
    }

    // Default reference date (1979-01-01 for ERA5 / ESDC)
    (1979, 1, 1, days_step)
}

fn parse_iso_date(s: &str) -> Option<(usize, usize, usize)> {
    let clean = s.replace('T', " ");
    let date_part = clean.split_whitespace().next().unwrap_or(&clean);
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() >= 3 {
        let y = parts[0].parse().ok()?;
        let m = parts[1].parse().ok()?;
        let d = parts[2].parse().ok()?;
        return Some((y, m, d));
    }
    None
}

/// Dynamically add N days to a starting date (year, month, day), handling month lengths and leap years.
pub fn add_days_to_date(
    start_year: usize,
    start_month: usize,
    start_day: usize,
    days_to_add: usize,
) -> (usize, usize, usize) {
    let mut year = start_year;
    let mut month = start_month;
    let mut day = start_day + days_to_add;

    loop {
        let days_in_cur_month = days_in_month(year, month);
        if day <= days_in_cur_month {
            break;
        }
        day -= days_in_cur_month;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }

    (year, month, day)
}

fn is_leap_year(year: usize) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: usize, month: usize) -> usize {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Format axis value dynamically based on dimension name, CF unit string, metadata time attributes, and step index.
pub fn format_axis_value(
    timestep: usize,
    max_timesteps: usize,
    dim_name: Option<&str>,
    units_str: Option<&str>,
    time_start: Option<&str>,
    temp_res: Option<&str>,
    target_hint: Option<&str>,
) -> String {
    let dim_lower = dim_name.unwrap_or("time").to_lowercase();
    let units_lower = units_str.unwrap_or("").to_lowercase();

    // 1. Dynamic Time / Date axis formatting
    if dim_lower.contains("time")
        || dim_lower.contains("date")
        || dim_lower.contains("year")
        || dim_lower.contains("month")
        || units_lower.contains("since")
        || units_lower.contains("day")
        || units_lower.contains("hour")
    {
        let (start_year, start_month, start_day, days_per_step) =
            parse_reference_date(units_str, time_start, temp_res, target_hint);
        let total_days_offset = timestep * days_per_step;
        let (year, month, day) =
            add_days_to_date(start_year, start_month, start_day, total_days_offset);

        return format!("{:04}-{:02}-{:02}", year, month, day);
    }

    // 2. Pressure levels (hPa, Pa, bar, plev, level)
    if dim_lower.contains("pres")
        || dim_lower.contains("level")
        || dim_lower.contains("plev")
        || units_lower.contains("hpa")
        || units_lower.contains("pa")
    {
        let hpa_value = 1000.0 - (timestep as f32 * 10.0).min(950.0);
        return format!("{:.0} hPa", hpa_value);
    }

    // 3. Spatial Coordinates (Degrees, Lat, Lon)
    if dim_lower.contains("lat")
        || dim_lower.contains("deg_n")
        || units_lower.contains("degrees_north")
    {
        let lat = -90.0 + (timestep as f32 * 2.5);
        let cardinal = if lat >= 0.0 { "N" } else { "S" };
        return format!("{:.1}° {}", lat.abs(), cardinal);
    }

    if dim_lower.contains("lon")
        || dim_lower.contains("deg_e")
        || units_lower.contains("degrees_east")
    {
        let lon = -180.0 + (timestep as f32 * 2.5);
        let cardinal = if lon >= 0.0 { "E" } else { "W" };
        return format!("{:.1}° {}", lon.abs(), cardinal);
    }

    // Fallback default
    format!("Step {} / {}", timestep + 1, max_timesteps)
}

/// Formats a numerical value with CF units or bare units into a human-readable location/duration/date string.
pub fn parse_loc(val: Option<f64>, units_str: &str) -> Option<String> {
    let v = val?;
    let lower_units = units_str.trim().to_lowercase();

    // 1. CF Absolute Datetime (e.g. "hours since 2024-01-01")
    if let Some((unit_part, ref_date_str)) = lower_units.split_once(" since ") {
        let (y, m, d) = parse_iso_date(ref_date_str.trim()).unwrap_or((1970, 1, 1));
        let scale_ms = unit_to_milliseconds(unit_part.trim()).unwrap_or(1_000) as f64;
        let total_ms = v * scale_ms;
        let total_hours = (total_ms / 3_600_000.0).round() as i64;

        let days_added = total_hours.div_euclid(24) as usize;
        let hour_of_day = total_hours.rem_euclid(24) as usize;

        let (res_y, res_m, res_d) = add_days_to_date(y, m, d, days_added);
        return Some(format!(
            "{:02}-{:02}-{:04} {:02}:00",
            res_m, res_d, res_y, hour_of_day
        ));
    }

    // 2. Bare Time Duration (e.g. "hours", "seconds", "d")
    if let Some(scale_ms) = unit_to_milliseconds(&lower_units) {
        let ms = v * scale_ms as f64;
        if ms == 0.0 {
            if lower_units.contains("sec")
                || lower_units == "s"
                || lower_units.contains("hour")
                || lower_units == "h"
                || lower_units == "hr"
                || lower_units == "hrs"
            {
                return Some("0 h".to_string());
            }
            return Some("0 ms".to_string());
        }

        // If unit is explicitly hour-based, keep in hours (e.g. 24 h -> "24 h")
        if lower_units == "h"
            || lower_units == "hr"
            || lower_units == "hrs"
            || lower_units == "hour"
            || lower_units == "hours"
        {
            return Some(format_num_with_unit(v, "h"));
        }
        // If unit is day-based, keep in days
        if lower_units == "d" || lower_units == "day" || lower_units == "days" {
            return Some(format_num_with_unit(v, "d"));
        }

        // For seconds or minutes, convert to coarsest unit
        if ms >= 3_600_000.0 && ms % 3_600_000.0 == 0.0 {
            let hours = ms / 3_600_000.0;
            return Some(format_num_with_unit(hours, "h"));
        }
        if ms >= 60_000.0 && ms % 60_000.0 == 0.0 {
            let mins = ms / 60_000.0;
            return Some(format_num_with_unit(mins, "min"));
        }
        if ms >= 1_000.0 && ms % 1_000.0 == 0.0 {
            let secs = ms / 1_000.0;
            return Some(format_num_with_unit(secs, "s"));
        }

        if lower_units.contains("min") {
            return Some(format_num_with_unit(v, "min"));
        }
        if lower_units.contains("sec") || lower_units == "s" {
            return Some(format_num_with_unit(v, "s"));
        }

        return Some(format_num_with_unit(ms, "ms"));
    }

    // 3. Degrees (e.g. "degrees_east", "deg")
    if lower_units.contains("deg") {
        return Some(format!("{:.2}°", v));
    }

    // 4. Default fallback
    Some(format!("{:.2}", v))
}

fn format_num_with_unit(v: f64, unit: &str) -> String {
    if v.fract() == 0.0 {
        format!("{:.0} {}", v, unit)
    } else {
        format!("{:.2} {}", v, unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_unit_hour_aliases() {
        let (s1, _) = parse_time_unit(Some("hours since 2024-01-01"));
        let (s2, _) = parse_time_unit(Some("h since 2024-01-01"));
        let (s3, _) = parse_time_unit(Some("hr since 2024-01-01"));
        let (s4, _) = parse_time_unit(Some("hrs since 2024-01-01"));

        assert_eq!(s1, 3600000);
        assert_eq!(s2, 3600000);
        assert_eq!(s3, 3600000);
        assert_eq!(s4, 3600000);
    }

    #[test]
    fn test_parse_time_unit_other_aliases() {
        assert_eq!(parse_time_unit(Some("min since 2024-01-01")).0, 60000);
        assert_eq!(parse_time_unit(Some("mins since 2024-01-01")).0, 60000);
        assert_eq!(parse_time_unit(Some("s since 2024-01-01")).0, 1000);
        assert_eq!(parse_time_unit(Some("sec since 2024-01-01")).0, 1000);
        assert_eq!(parse_time_unit(Some("secs since 2024-01-01")).0, 1000);
        assert_eq!(parse_time_unit(Some("d since 2024-01-01")).0, 86400000);
        assert_eq!(parse_time_unit(Some("ms since 2024-01-01")).0, 1);
    }

    #[test]
    fn test_parse_time_unit_bare_duration() {
        assert_eq!(parse_time_unit(Some("hours")).0, 3600000);
        assert_eq!(parse_time_unit(Some("hour")).0, 3600000);
        assert_eq!(parse_time_unit(Some("h")).0, 3600000);
        assert_eq!(parse_time_unit(Some("hr")).0, 3600000);
        assert_eq!(parse_time_unit(Some("hrs")).0, 3600000);
        assert_eq!(parse_time_unit(Some("days")).0, 86400000);
        assert_eq!(parse_time_unit(Some("minutes")).0, 60000);
    }

    #[test]
    fn test_parse_loc_durations() {
        assert_eq!(parse_loc(Some(12.0), "hours"), Some("12 h".to_string()));
        assert_eq!(parse_loc(Some(24.0), "h"), Some("24 h".to_string()));
        assert_eq!(parse_loc(Some(6.0), "hr"), Some("6 h".to_string()));
        assert_eq!(parse_loc(Some(48.0), "hrs"), Some("48 h".to_string()));
        assert_eq!(parse_loc(Some(0.0), "hour"), Some("0 h".to_string()));
        assert_eq!(parse_loc(Some(12.5), "hours"), Some("12.50 h".to_string()));

        assert_eq!(parse_loc(Some(0.0), "seconds"), Some("0 h".to_string()));
        assert_eq!(parse_loc(Some(3600.0), "seconds"), Some("1 h".to_string()));
        assert_eq!(parse_loc(Some(7200.0), "s"), Some("2 h".to_string()));
        assert_eq!(parse_loc(Some(10800.0), "sec"), Some("3 h".to_string()));
        assert_eq!(parse_loc(Some(30.0), "seconds"), Some("30 s".to_string()));
        assert_eq!(
            parse_loc(Some(1800.0), "seconds"),
            Some("30 min".to_string())
        );
        assert_eq!(parse_loc(Some(5.0), "d"), Some("5 d".to_string()));
        assert_eq!(parse_loc(Some(500.0), "ms"), Some("500 ms".to_string()));
    }

    #[test]
    fn test_parse_loc_datetime() {
        assert_eq!(
            parse_loc(Some(12.0), "hours since 2024-01-01"),
            Some("01-01-2024 12:00".to_string())
        );
        assert_eq!(
            parse_loc(Some(12.0), "h since 2024-01-01"),
            Some("01-01-2024 12:00".to_string())
        );
        assert_eq!(
            parse_loc(Some(12.0), "hrs since 2024-01-01"),
            Some("01-01-2024 12:00".to_string())
        );
    }

    #[test]
    fn test_parse_loc_degrees_and_fallback() {
        assert_eq!(
            parse_loc(Some(-120.5), "degrees_east"),
            Some("-120.50°".to_string())
        );
        assert_eq!(parse_loc(Some(45.0), "deg"), Some("45.00°".to_string()));
        assert_eq!(parse_loc(Some(100.0), "hPa"), Some("100.00".to_string()));
        assert_eq!(parse_loc(None, "hours"), None);
    }
}
