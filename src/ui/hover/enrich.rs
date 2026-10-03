//! Tooltip dimension enrichment for animated and collapsed dimensions.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, DimensionSelection, VariableInfo};
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;
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
    entries: &mut Vec<HoverField>,
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
