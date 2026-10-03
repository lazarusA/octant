//! Unified coordinate, datetime, and physical dimension formatting engine.

use super::calendar::add_days_to_date;
use super::cf::{
    is_cf_time_unit, parse_iso_date, parse_reference_date, split_since, unit_to_milliseconds,
};
use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// Formats a coordinate scalar value (without the dimension name) with units, datetime
/// conversion, or cardinal degrees.
pub fn format_scalar_coordinate(
    dim_name: &str,
    val: f64,
    units: Option<&str>,
    time_start: Option<&str>,
    is_time_dim: bool,
) -> String {
    let clean = dim_name.trim();

    // 1. CF Relative Date/Time or Duration formatting
    if let Some(u) = units
        && is_cf_time_unit(u)
        && let Some(formatted) = parse_loc(Some(val), u)
    {
        return formatted;
    }

    // 2. ISO reference start date (e.g. from time_coverage_start attribute)
    if is_time_dim
        && let Some(start_str) = time_start
        && let Some((y, m, d)) = parse_iso_date(start_str)
    {
        let (res_y, res_m, res_d) = add_days_to_date(y, m, d, val.round() as i64);
        return format!("{:04}-{:02}-{:02}", res_y, res_m, res_d);
    }

    // 3. Unix timestamps (> 100M seconds or > 100B ms)
    if is_time_dim && val.abs() > 1e8 && val.abs() < 1e13 {
        let total_secs = if val.abs() > 1e11 {
            (val / 1000.0).round() as i64
        } else {
            val.round() as i64
        };
        let total_hours = total_secs.div_euclid(3600);
        let days_added = total_hours.div_euclid(24);
        let hour_of_day = total_hours.rem_euclid(24) as usize;
        let (res_y, res_m, res_d) = add_days_to_date(1970, 1, 1, days_added);
        return format!(
            "{:04}-{:02}-{:02} {:02}:00",
            res_y, res_m, res_d, hour_of_day
        );
    }

    // 4. Step indices
    if is_time_dim && (clean.eq_ignore_ascii_case("step") || clean.eq_ignore_ascii_case("timestep"))
    {
        return format!("t={:.0}", val);
    }

    // 5. Physical & spatial coordinate scalar formatting
    format_coord_scalar(clean, val, units)
}

/// Formats a spatial or physical scalar with standard symbol suffixes (cardinal degrees, hPa, m).
pub fn format_coord_scalar(clean: &str, val: f64, units: Option<&str>) -> String {
    if contains_ascii_case_insensitive(clean, "lon") {
        format_cardinal_degrees(val, true)
    } else if contains_ascii_case_insensitive(clean, "lat") {
        format_cardinal_degrees(val, false)
    } else if let Some(u) = units
        && !u.trim().is_empty()
        && !u.eq_ignore_ascii_case("1")
        && !u.eq_ignore_ascii_case("none")
        && !u.eq_ignore_ascii_case("dimensionless")
    {
        format!("{:.2} {}", val, u.trim())
    } else if contains_ascii_case_insensitive(clean, "depth")
        || contains_ascii_case_insensitive(clean, "height")
        || contains_ascii_case_insensitive(clean, "alt")
    {
        format!("{:.2} m", val)
    } else if contains_ascii_case_insensitive(clean, "level")
        || contains_ascii_case_insensitive(clean, "lev")
        || contains_ascii_case_insensitive(clean, "plev")
        || contains_ascii_case_insensitive(clean, "pressure")
        || contains_ascii_case_insensitive(clean, "pres")
    {
        format!("{:.2} hPa", val)
    } else {
        format!("{:.2}", val)
    }
}

/// Formats latitude or longitude into cardinal degree string (e.g. `122.40°W`, `37.70°N`).
#[inline]
pub fn format_cardinal_degrees(val: f64, is_longitude: bool) -> String {
    let suffix = if is_longitude {
        if val >= 0.0 { "°E" } else { "°W" }
    } else if val >= 0.0 {
        "°N"
    } else {
        "°S"
    };
    format!("{:.2}{}", val.abs(), suffix)
}

/// Formats a numerical value with CF units or bare units into a human-readable location/duration/date string.
pub fn parse_loc(val: Option<f64>, units_str: &str) -> Option<String> {
    let v = val?;
    let clean_units = units_str.trim();

    // 1. CF Absolute Datetime (e.g. "hours since 2024-01-01")
    if let Some((unit_part, ref_date_str)) = split_since(clean_units) {
        let (y, m, d) = parse_iso_date(ref_date_str).unwrap_or((1970, 1, 1));
        let scale_ms = unit_to_milliseconds(unit_part).unwrap_or(1_000) as f64;
        let total_ms = v * scale_ms;
        let total_hours = (total_ms / 3_600_000.0).round() as i64;
        let days_added = total_hours.div_euclid(24);
        let hour_of_day = total_hours.rem_euclid(24) as usize;

        let (res_y, res_m, res_d) = add_days_to_date(y, m, d, days_added);
        return Some(format!(
            "{:02}-{:02}-{:04} {:02}:00",
            res_m, res_d, res_y, hour_of_day
        ));
    }

    // 2. Bare Time Duration (e.g. "hours", "seconds", "d")
    if let Some(scale_ms) = unit_to_milliseconds(clean_units) {
        let ms = v * scale_ms as f64;
        if ms == 0.0 {
            if contains_ascii_case_insensitive(clean_units, "sec")
                || clean_units.eq_ignore_ascii_case("s")
                || contains_ascii_case_insensitive(clean_units, "hour")
                || clean_units.eq_ignore_ascii_case("h")
                || clean_units.eq_ignore_ascii_case("hr")
                || clean_units.eq_ignore_ascii_case("hrs")
            {
                return Some("0 h".to_string());
            }
            return Some("0 ms".to_string());
        }

        if clean_units.eq_ignore_ascii_case("h")
            || clean_units.eq_ignore_ascii_case("hr")
            || clean_units.eq_ignore_ascii_case("hrs")
            || clean_units.eq_ignore_ascii_case("hour")
            || clean_units.eq_ignore_ascii_case("hours")
        {
            return Some(format_num_with_unit(v, "h"));
        }
        if clean_units.eq_ignore_ascii_case("d")
            || clean_units.eq_ignore_ascii_case("day")
            || clean_units.eq_ignore_ascii_case("days")
        {
            return Some(format_num_with_unit(v, "d"));
        }

        // For seconds or minutes, convert to coarsest unit
        if ms >= 3_600_000.0 && ms % 3_600_000.0 == 0.0 {
            return Some(format_num_with_unit(ms / 3_600_000.0, "h"));
        }
        if ms >= 60_000.0 && ms % 60_000.0 == 0.0 {
            return Some(format_num_with_unit(ms / 60_000.0, "min"));
        }
        if ms >= 1_000.0 && ms % 1_000.0 == 0.0 {
            return Some(format_num_with_unit(ms / 1_000.0, "s"));
        }

        if contains_ascii_case_insensitive(clean_units, "min") {
            return Some(format_num_with_unit(v, "min"));
        }
        if contains_ascii_case_insensitive(clean_units, "sec")
            || clean_units.eq_ignore_ascii_case("s")
        {
            return Some(format_num_with_unit(v, "s"));
        }

        return Some(format_num_with_unit(ms, "ms"));
    }

    // 3. Degrees (e.g. "degrees_east", "deg")
    if contains_ascii_case_insensitive(clean_units, "deg") {
        return Some(format!("{:.2}°", v));
    }

    // 4. Default fallback
    Some(format!("{:.2}", v))
}

/// Helper to format number with unit, rendering integer without trailing decimals.
#[inline]
pub fn format_num_with_unit(v: f64, unit: &str) -> String {
    if v.fract() == 0.0 {
        format!("{:.0} {}", v, unit)
    } else {
        format!("{:.2} {}", v, unit)
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
    let dim = dim_name.unwrap_or("time");
    let units = units_str.unwrap_or("");

    // 1. Dynamic Time / Date axis formatting
    if contains_ascii_case_insensitive(dim, "time")
        || contains_ascii_case_insensitive(dim, "date")
        || contains_ascii_case_insensitive(dim, "year")
        || contains_ascii_case_insensitive(dim, "month")
        || contains_ascii_case_insensitive(units, "since")
        || contains_ascii_case_insensitive(units, "day")
        || contains_ascii_case_insensitive(units, "hour")
    {
        let (start_year, start_month, start_day, days_per_step) =
            parse_reference_date(units_str, time_start, temp_res, target_hint);
        let total_days_offset = (timestep * days_per_step) as i64;
        let (year, month, day) =
            add_days_to_date(start_year, start_month, start_day, total_days_offset);

        return format!("{:04}-{:02}-{:02}", year, month, day);
    }

    // 2. Pressure levels (hPa, Pa, bar, plev, level)
    if contains_ascii_case_insensitive(dim, "pres")
        || contains_ascii_case_insensitive(dim, "level")
        || contains_ascii_case_insensitive(dim, "plev")
        || contains_ascii_case_insensitive(units, "hpa")
        || contains_ascii_case_insensitive(units, "pa")
    {
        let hpa_value = 1000.0 - (timestep as f32 * 10.0).min(950.0);
        return format!("{:.0} hPa", hpa_value);
    }

    // 3. Spatial Coordinates (Degrees, Lat, Lon)
    if contains_ascii_case_insensitive(dim, "lat")
        || contains_ascii_case_insensitive(dim, "deg_n")
        || contains_ascii_case_insensitive(units, "degrees_north")
    {
        let lat = -90.0 + (timestep as f64 * 2.5);
        return format_cardinal_degrees(lat, false);
    }

    if contains_ascii_case_insensitive(dim, "lon")
        || contains_ascii_case_insensitive(dim, "deg_e")
        || contains_ascii_case_insensitive(units, "degrees_east")
    {
        let lon = -180.0 + (timestep as f64 * 2.5);
        return format_cardinal_degrees(lon, true);
    }

    // Fallback default
    format!("Step {} / {}", timestep + 1, max_timesteps)
}
