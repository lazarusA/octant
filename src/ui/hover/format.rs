//! Tooltip dimension coordinate formatting with physical units.

use crate::data::{DatasetMetadata, VariableInfo};

/// Formats dimension coordinate values with physical units, cardinal degrees, pressure, or datetime.
pub fn format_dimension_coord(
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    store_target: Option<&str>,
    dim_name: &str,
    idx: usize,
    total_len: usize,
    geo_fallback: Option<f32>,
) -> String {
    let clean = dim_name.trim().to_lowercase();

    // Look up dedicated coordinate variable info from metadata if available
    let coord_var = meta.and_then(|m| {
        m.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(dim_name))
    });

    let dim_units = coord_var
        .and_then(|cv| {
            cv.units
                .as_deref()
                .or(cv.attributes.get("units").map(|s| s.as_str()))
        })
        .or_else(|| {
            if var.is_some_and(|v| v.name.eq_ignore_ascii_case(dim_name)) {
                var.and_then(|v| {
                    v.units
                        .as_deref()
                        .or(v.attributes.get("units").map(|s| s.as_str()))
                })
            } else {
                None
            }
        });

    let is_time_dim = crate::data::coordinates::naming::is_animated_time_name(dim_name)
        || dim_units.is_some_and(|u| {
            let l = u.to_lowercase();
            l.contains("since")
                || l.contains("hour")
                || l.contains("day")
                || l.contains("sec")
                || l.contains("min")
                || l.contains("year")
                || l.contains("month")
        });

    let time_start = coord_var
        .and_then(|cv| {
            cv.time_coverage_start
                .as_deref()
                .or(cv.attributes.get("time_coverage_start").map(|s| s.as_str()))
        })
        .or_else(|| {
            var.and_then(|v| {
                v.time_coverage_start
                    .as_deref()
                    .or(v.attributes.get("time_coverage_start").map(|s| s.as_str()))
            })
        });

    let temp_res = coord_var
        .and_then(|cv| {
            cv.temporal_resolution
                .as_deref()
                .or(cv.attributes.get("temporal_resolution").map(|s| s.as_str()))
        })
        .or_else(|| {
            var.and_then(|v| {
                v.temporal_resolution
                    .as_deref()
                    .or(v.attributes.get("temporal_resolution").map(|s| s.as_str()))
            })
        });

    if let Some(m) = meta {
        if let Some(coords) = m.get_dim_coords(var.map(|v| v.name.as_str()), dim_name) {
            if coords.len() == total_len
                && let Some(c) = coords.get(idx)
                && !c.trim().is_empty()
            {
                let is_raw_numeric = c.parse::<f64>().is_ok()
                    && !c.contains('-')
                    && !c.contains(':')
                    && !c.contains('/')
                    && !c.contains('T');

                if !is_raw_numeric && !c.trim().is_empty() {
                    return format!("{}:\u{00A0}{}", dim_name, c.replace(' ', "\u{00A0}"));
                }

                if let Ok(val) = c.parse::<f64>() {
                    return format_coord_value(dim_name, val, dim_units, time_start, is_time_dim);
                }
            }

            if coords.len() >= 2
                && let (Some(first), Some(last)) = (coords.first(), coords.last())
            {
                let first_is_date =
                    first.contains('-') || first.contains(':') || first.contains('T');
                if is_time_dim && first_is_date {
                    let time_val = crate::utils::units::format_axis_value(
                        idx,
                        total_len,
                        Some(dim_name),
                        dim_units,
                        Some(first.as_str()),
                        temp_res,
                        store_target,
                    );
                    return format!("{}:\u{00A0}{}", dim_name, time_val);
                }

                if let (Ok(f_v), Ok(l_v)) = (first.parse::<f64>(), last.parse::<f64>()) {
                    let t = if total_len > 1 {
                        idx as f64 / (total_len - 1) as f64
                    } else {
                        0.0
                    };
                    let val = f_v + t * (l_v - f_v);
                    return format_coord_value(dim_name, val, dim_units, time_start, is_time_dim);
                }
            }

            if is_time_dim {
                let time_val = crate::utils::units::format_axis_value(
                    idx,
                    total_len,
                    Some(dim_name),
                    dim_units,
                    time_start,
                    temp_res,
                    store_target,
                );
                return format!("{}:\u{00A0}{}", dim_name, time_val);
            }

            if let Some(first) = coords.first()
                && !first.trim().is_empty()
            {
                return format!("{}:\u{00A0}{}", dim_name, first.replace(' ', "\u{00A0}"));
            }
        }

        if let Some((min_b, max_b)) =
            m.get_coord_bounds_for_var(var.map(|v| v.name.as_str()), dim_name)
        {
            let t = if total_len > 1 {
                idx as f64 / (total_len - 1) as f64
            } else {
                0.0
            };
            let val = min_b + t * (max_b - min_b);
            return format_coord_value(dim_name, val, dim_units, time_start, is_time_dim);
        }
    }

    if let Some(geo) = geo_fallback {
        return if clean.contains("lon") {
            let cardinal = if geo >= 0.0 { "°E" } else { "°W" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else if clean.contains("lat") {
            let cardinal = if geo >= 0.0 { "°N" } else { "°S" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else {
            format!("{}:\u{00A0}{:.2}°", dim_name, geo)
        };
    }

    if is_time_dim {
        let time_val = crate::utils::units::format_axis_value(
            idx,
            total_len,
            Some(dim_name),
            dim_units,
            time_start,
            temp_res,
            store_target,
        );
        return format!("{}:\u{00A0}{}", dim_name, time_val);
    }

    if total_len > 1 {
        format!("{}:\u{00A0}{}/{}", dim_name, idx + 1, total_len)
    } else {
        format!("{}:\u{00A0}{}", dim_name, idx)
    }
}

fn format_coord_value(
    dim_name: &str,
    val: f64,
    units: Option<&str>,
    time_start: Option<&str>,
    is_time_dim: bool,
) -> String {
    let clean = dim_name.trim().to_lowercase();

    // 1. CF Relative Date/Time or Duration formatting via parse_loc
    if let Some(u) = units {
        let lower = u.to_lowercase();
        let is_cf_time = lower.contains("since")
            || lower.contains("hour")
            || lower.contains("day")
            || lower.contains("sec")
            || lower.contains("min")
            || lower.contains("ms")
            || lower == "h"
            || lower == "d"
            || lower == "s"
            || lower == "hr"
            || lower == "hrs";

        if is_cf_time && let Some(formatted) = crate::utils::units::parse_loc(Some(val), u) {
            return format!("{}:\u{00A0}{}", dim_name, formatted);
        }
    }

    // 2. ISO reference start date (e.g. from time_coverage_start attribute)
    if is_time_dim {
        if let Some(start_str) = time_start
            && let Some((y, m, d)) = crate::utils::units::parse_iso_date(start_str.trim())
        {
            let (res_y, res_m, res_d) =
                crate::utils::units::add_days_to_date(y, m, d, val.round() as i64);
            return format!(
                "{}:\u{00A0}{:04}-{:02}-{:02}",
                dim_name, res_y, res_m, res_d
            );
        }

        // Unix timestamps (> 100M seconds or > 100B ms)
        if val.abs() > 1e8 && val.abs() < 1e13 {
            let total_secs = if val.abs() > 1e11 {
                (val / 1000.0).round() as i64
            } else {
                val.round() as i64
            };
            let total_hours = total_secs.div_euclid(3600);
            let days_added = total_hours.div_euclid(24);
            let hour_of_day = total_hours.rem_euclid(24) as usize;
            let (res_y, res_m, res_d) =
                crate::utils::units::add_days_to_date(1970, 1, 1, days_added);
            return format!(
                "{}:\u{00A0}{:04}-{:02}-{:02} {:02}:00",
                dim_name, res_y, res_m, res_d, hour_of_day
            );
        }

        if clean == "step" || clean == "timestep" {
            return format!("{}:\u{00A0}t={:.0}", dim_name, val);
        }
    }

    // 3. Physical & spatial coordinate scalar formatting
    format_coord_scalar(&clean, dim_name, val, units)
}

fn format_coord_scalar(clean: &str, dim_name: &str, val: f64, units: Option<&str>) -> String {
    if clean.contains("lon") {
        let cardinal = if val >= 0.0 { "°E" } else { "°W" };
        format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
    } else if clean.contains("lat") {
        let cardinal = if val >= 0.0 { "°N" } else { "°S" };
        format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
    } else if let Some(u) = units
        && !u.trim().is_empty()
        && u != "1"
        && u != "none"
        && u != "dimensionless"
    {
        format!("{}:\u{00A0}{:.2}\u{00A0}{}", dim_name, val, u.trim())
    } else if clean.contains("depth") || clean.contains("height") || clean.contains("alt") {
        format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
    } else if clean.contains("level")
        || clean.contains("lev")
        || clean.contains("plev")
        || clean.contains("pressure")
        || clean.contains("pres")
    {
        format!("{}:\u{00A0}{:.2}\u{00A0}hPa", dim_name, val)
    } else {
        format!("{}:\u{00A0}{:.2}", dim_name, val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_coord_value_cf_time() {
        let res = format_coord_value("time", 24.0, Some("hours since 2024-01-01"), None, true);
        assert_eq!(res, "time:\u{00A0}01-02-2024 00:00");
    }

    #[test]
    fn test_format_coord_value_durations() {
        let res_h = format_coord_value("time", 12.0, Some("hours"), None, true);
        assert_eq!(res_h, "time:\u{00A0}12 h");

        let res_d = format_coord_value("leadtime", 7.0, Some("days"), None, true);
        assert_eq!(res_d, "leadtime:\u{00A0}7 d");
    }

    #[test]
    fn test_format_coord_value_cardinals() {
        let res_lon = format_coord_value("lon", -122.4, None, None, false);
        assert_eq!(res_lon, "lon:\u{00A0}122.40°W");

        let res_lat = format_coord_value("lat", 37.7, None, None, false);
        assert_eq!(res_lat, "lat:\u{00A0}37.70°N");
    }

    #[test]
    fn test_format_coord_value_step() {
        let res_step = format_coord_value("step", 42.0, None, None, true);
        assert_eq!(res_step, "step:\u{00A0}t=42");
    }

    #[test]
    fn test_format_coord_value_custom_units() {
        let res = format_coord_value("elevation", 1500.0, Some("m"), None, false);
        assert_eq!(res, "elevation:\u{00A0}1500.00\u{00A0}m");
    }
}
