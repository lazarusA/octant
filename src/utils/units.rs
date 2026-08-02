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
    let singular = if clean_unit.ends_with('s') {
        clean_unit.trim_end_matches('s')
    } else {
        &clean_unit
    };

    match singular {
        "millisecond" | "msec" | "ms" => Some(1),
        "second" | "sec" | "s" => Some(1_000),
        "minute" | "min" => Some(60 * 1_000),
        "hour" | "hr" | "h" => Some(60 * 60 * 1_000),
        "day" | "d" => Some(24 * 60 * 60 * 1_000),
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
    // 1. Try explicit time_coverage_start (e.g. "1979-01-01T00:00:00")
    let ref_date = time_start.and_then(parse_iso_date);

    // 2. Try temporal_resolution (e.g. "8D" -> 8, "16D" -> 16, "1D" -> 1)
    let days_step = if let Some(res) = temp_res {
        let clean = res.trim().to_uppercase();
        if clean.ends_with('D') {
            clean.trim_end_matches('D').parse::<usize>().unwrap_or(8)
        } else if clean.ends_with("DAY") || clean.ends_with("DAYS") {
            clean.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(1)
        } else {
            8
        }
    } else if let Some(target) = target_hint {
        if target.contains("16d") {
            16
        } else if target.contains("8d") {
            8
        } else if target.contains("1d") || target.contains("daily") {
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
                return (y, m, d, days_step);
            }
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
pub fn add_days_to_date(start_year: usize, start_month: usize, start_day: usize, days_to_add: usize) -> (usize, usize, usize) {
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
        let (start_year, start_month, start_day, days_per_step) = parse_reference_date(units_str, time_start, temp_res, target_hint);
        let total_days_offset = timestep * days_per_step;
        let (year, month, day) = add_days_to_date(start_year, start_month, start_day, total_days_offset);

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
