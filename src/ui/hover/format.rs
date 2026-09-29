//! Tooltip dimension coordinate formatting with physical units.

use crate::data::coordinates::naming::{contains_ascii_case_insensitive, is_animated_time_name};
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::units::{
    format_axis_value, format_cardinal_degrees, format_scalar_coordinate, is_cf_time_unit,
};

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

    let is_time_dim = is_animated_time_name(dim_name) || dim_units.is_some_and(is_cf_time_unit);

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

                if !is_raw_numeric {
                    return format!("{}:\u{00A0}{}", dim_name, c.replace(' ', "\u{00A0}"));
                }

                if let Ok(val) = c.parse::<f64>() {
                    return format_scalar_coordinate(
                        dim_name,
                        val,
                        dim_units,
                        time_start,
                        is_time_dim,
                    );
                }
            }

            if coords.len() >= 2
                && let (Some(first), Some(last)) = (coords.first(), coords.last())
            {
                let first_is_date =
                    first.contains('-') || first.contains(':') || first.contains('T');
                if is_time_dim && first_is_date {
                    let time_val = format_axis_value(
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
                    return format_scalar_coordinate(
                        dim_name,
                        val,
                        dim_units,
                        time_start,
                        is_time_dim,
                    );
                }
            }

            if is_time_dim {
                let time_val = format_axis_value(
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
            return format_scalar_coordinate(dim_name, val, dim_units, time_start, is_time_dim);
        }
    }

    if let Some(geo) = geo_fallback {
        return if contains_ascii_case_insensitive(dim_name, "lon") {
            format!(
                "{}:\u{00A0}{}",
                dim_name,
                format_cardinal_degrees(geo as f64, true)
            )
        } else if contains_ascii_case_insensitive(dim_name, "lat") {
            format!(
                "{}:\u{00A0}{}",
                dim_name,
                format_cardinal_degrees(geo as f64, false)
            )
        } else {
            format!("{}:\u{00A0}{:.2}°", dim_name, geo)
        };
    }

    if is_time_dim {
        let time_val = format_axis_value(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_dimension_coord_fallback() {
        let res_dim = format_dimension_coord(None, None, None, "dim0", 5, 10, None);
        assert_eq!(res_dim, "dim0:\u{00A0}6/10");

        let res_step = format_dimension_coord(None, None, None, "step", 5, 10, None);
        assert_eq!(res_step, "step:\u{00A0}Step 6 / 10");
    }

    #[test]
    fn test_format_dimension_coord_geo_fallback() {
        let res_lon = format_dimension_coord(None, None, None, "lon", 0, 1, Some(-45.5));
        assert_eq!(res_lon, "lon:\u{00A0}45.50°W");

        let res_lat = format_dimension_coord(None, None, None, "lat", 0, 1, Some(12.25));
        assert_eq!(res_lat, "lat:\u{00A0}12.25°N");
    }
}
