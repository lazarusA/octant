//! Tooltip dimension coordinate formatting and enrichment.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, DimensionSelection, VariableInfo};
use std::collections::HashSet;

/// Resolves the global dataset origin and full length for a dimension index `dim_idx`.
pub fn get_dimension_origin_and_full_len(
    app: &OctantApp,
    var: Option<&VariableInfo>,
    dim_idx: usize,
) -> (usize, usize) {
    let full_len = var.and_then(|v| v.shape.get(dim_idx)).copied().unwrap_or(1) as usize;

    let origin = if let Some(req) = &app.active_slice_request
        && let Some(sel) = req.selections.get(dim_idx)
    {
        match sel {
            DimensionSelection::Range { start, .. } => *start,
            DimensionSelection::Index(idx) => *idx,
        }
    } else {
        app.plotted_selected_dim_ranges
            .get(dim_idx)
            .or_else(|| app.selected_dim_ranges.get(dim_idx))
            .map(|(start, _)| *start)
            .unwrap_or(0)
    };

    (origin, full_len)
}

/// Appends/prepends the Animated dimension value and all Collapsed dimension values to entries.
pub fn enrich_entries_with_animated_and_collapsed_dims(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    entries: &mut Vec<String>,
    used_dims: &mut HashSet<usize>,
) {
    let Some(v) = var else { return };
    if v.dimension_names.is_empty() {
        return;
    }

    // 1. Identify Animated Dimension
    let anim_dim = app
        .plotted_animated_dim
        .or(app.animated_dim)
        .or_else(|| {
            let configs = if !app.plotted_dim_config.is_empty() {
                &app.plotted_dim_config
            } else {
                &app.dim_config
            };
            configs
                .iter()
                .position(|c| c.animation == crate::app::AnimationRole::Animated)
        })
        .or_else(|| {
            if v.shape.len() >= 3 && !used_dims.contains(&0) {
                Some(0)
            } else {
                None
            }
        });

    let mut has_animated_inserted = false;
    if let Some(a_idx) = anim_dim
        && a_idx < v.dimension_names.len()
        && !used_dims.contains(&a_idx)
    {
        let total_steps = app
            .animated_dim_extent()
            .max(v.shape.get(a_idx).copied().unwrap_or(1) as usize);
        let dim_name = &v.dimension_names[a_idx];
        let loc_anim = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            dim_name,
            app.current_timestep.min(total_steps.saturating_sub(1)),
            total_steps,
            None,
        );
        entries.insert(0, loc_anim);
        used_dims.insert(a_idx);
        has_animated_inserted = true;
    }

    // 2. Insert all remaining Collapsed Dimensions
    for d in 0..v.dimension_names.len() {
        if !used_dims.contains(&d) {
            let sel_idx = app
                .plotted_selected_dim_indices
                .get(d)
                .or_else(|| app.selected_dim_indices.get(d))
                .copied()
                .unwrap_or(0);
            let total_len = v.shape.get(d).copied().unwrap_or(1) as usize;
            let dim_name = &v.dimension_names[d];
            let loc_collapsed = format_dimension_coord(
                meta,
                Some(v),
                Some(&app.plotted_store_target_input),
                dim_name,
                sel_idx.min(total_len.saturating_sub(1)),
                total_len,
                None,
            );
            let insert_pos = if has_animated_inserted { 1 } else { 0 };
            entries.insert(insert_pos.min(entries.len()), loc_collapsed);
            used_dims.insert(d);
        }
    }
}

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

    let is_time_dim =
        clean.contains("time") || clean == "t" || clean.contains("date") || clean.contains("step");

    let units_str = var.and_then(|v| {
        v.units
            .as_deref()
            .or(v.attributes.get("units").map(|s| s.as_str()))
    });
    let time_start = var.and_then(|v| {
        v.time_coverage_start
            .as_deref()
            .or(v.attributes.get("time_coverage_start").map(|s| s.as_str()))
    });
    let temp_res = var.and_then(|v| {
        v.temporal_resolution
            .as_deref()
            .or(v.attributes.get("temporal_resolution").map(|s| s.as_str()))
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

                if is_time_dim {
                    let time_val = crate::utils::units::format_axis_value(
                        idx,
                        total_len,
                        Some(dim_name),
                        units_str,
                        time_start.or(Some(c.as_str())),
                        temp_res,
                        store_target,
                    );
                    return format!("{}:\u{00A0}{}", dim_name, time_val);
                }

                if let Ok(val) = c.parse::<f64>() {
                    return format_coord_scalar(&clean, dim_name, val);
                }
            }

            if coords.len() >= 2
                && let (Some(first), Some(last)) = (coords.first(), coords.last())
            {
                if is_time_dim {
                    let first_is_date =
                        first.contains('-') || first.contains(':') || first.contains('T');
                    let time_start_override = if first_is_date {
                        Some(first.as_str())
                    } else {
                        time_start
                    };
                    let time_val = crate::utils::units::format_axis_value(
                        idx,
                        total_len,
                        Some(dim_name),
                        units_str,
                        time_start_override,
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
                    return format_coord_scalar(&clean, dim_name, val);
                }
            }

            if is_time_dim {
                let time_val = crate::utils::units::format_axis_value(
                    idx,
                    total_len,
                    Some(dim_name),
                    units_str,
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
            if is_time_dim {
                let time_val = crate::utils::units::format_axis_value(
                    idx,
                    total_len,
                    Some(dim_name),
                    units_str,
                    time_start,
                    temp_res,
                    store_target,
                );
                return format!("{}:\u{00A0}{}", dim_name, time_val);
            }

            let t = if total_len > 1 {
                idx as f64 / (total_len - 1) as f64
            } else {
                0.0
            };
            let val = min_b + t * (max_b - min_b);
            return format_coord_scalar(&clean, dim_name, val);
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
            units_str,
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

fn format_coord_scalar(clean: &str, dim_name: &str, val: f64) -> String {
    if clean.contains("lon") {
        let cardinal = if val >= 0.0 { "°E" } else { "°W" };
        format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
    } else if clean.contains("lat") {
        let cardinal = if val >= 0.0 { "°N" } else { "°S" };
        format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
    } else if clean.contains("depth") || clean.contains("height") || clean.contains("alt") {
        format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
    } else if clean.contains("level")
        || clean.contains("lev")
        || clean.contains("plev")
        || clean.contains("pressure")
    {
        format!("{}:\u{00A0}{:.2}\u{00A0}hPa", dim_name, val)
    } else {
        format!("{}:\u{00A0}{:.2}", dim_name, val)
    }
}
