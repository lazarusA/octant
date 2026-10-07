use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
    stored_offset,
};
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;
use crate::ui::hover::sample_1d::sample_line_series;
use std::collections::HashSet;

pub(crate) fn resolve_line_plot_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    norm_x: f32,
    norm_y: f32,
) -> (f32, Vec<HoverField>, usize, usize) {
    let (profile_values, profile_length, line_count) = app.get_line_profile_payload();
    let prof_len = profile_length as usize;
    let l_count = line_count as usize;

    let (sample_idx, best_line_idx, val) =
        sample_line_series(app, norm_x, norm_y, &profile_values, prof_len, l_count);
    let mut used_dims = HashSet::new();

    let (dim_name, prof_dim_idx) = resolve_line_profile_dim(app, var);
    let (origin_prof, full_prof_len) = if let Some(idx) = prof_dim_idx {
        used_dims.insert(idx);
        get_dimension_origin_and_full_len(app, var, idx)
    } else {
        (0, prof_len)
    };
    let sample_offset = stored_offset(app, &dim_name, sample_idx, prof_len);
    let global_sample = (origin_prof + sample_offset).min(full_prof_len.saturating_sub(1));

    let loc_str = format_dimension_coord(
        meta,
        var,
        Some(&app.plotted_store_target_input),
        &dim_name,
        global_sample,
        full_prof_len,
        None,
    );
    let mut entries = vec![loc_str];

    if l_count > 1 {
        enrich_line_series_ortho_dim(
            app,
            meta,
            var,
            &mut entries,
            &mut used_dims,
            best_line_idx,
            l_count,
        );
    }

    enrich_entries_with_animated_and_collapsed_dims(app, meta, var, &mut entries, &mut used_dims);

    (val, entries, sample_idx, best_line_idx)
}

fn resolve_line_profile_dim(
    app: &OctantApp,
    var: Option<&VariableInfo>,
) -> (String, Option<usize>) {
    if let Some(v) = var {
        let (explicit_x, explicit_y, explicit_z) =
            v.resolve_spatial_dim_indices(app.effective_dim_config());

        let p_idx = match app.line_profile_dim_idx {
            0 => explicit_x.or_else(|| v.dimension_names.len().checked_sub(1)),
            1 => explicit_y.or_else(|| v.dimension_names.len().checked_sub(2)),
            _ => explicit_z.or_else(|| {
                (0..v.dimension_names.len())
                    .find(|&i| Some(i) != explicit_x && Some(i) != explicit_y)
            }),
        };

        let name = p_idx
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(app.line_profile_dim_idx))
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });

        (name, p_idx)
    } else {
        let name = app
            .get_spatial_dim_name(app.line_profile_dim_idx)
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });
        (name, None)
    }
}

fn enrich_line_series_ortho_dim(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    entries: &mut Vec<HoverField>,
    used_dims: &mut HashSet<usize>,
    best_line_idx: usize,
    l_count: usize,
) {
    if let Some(v) = var {
        let (explicit_x, explicit_y, _) = v.resolve_spatial_dim_indices(app.effective_dim_config());

        let ortho_dim_idx = match app.line_profile_dim_idx {
            0 => explicit_y,
            1 => explicit_x,
            _ => None,
        };

        if let Some(o_idx) = ortho_dim_idx
            && let Some(ortho_name) = v.dimension_names.get(o_idx)
        {
            let (origin_ortho, full_ortho_len) =
                get_dimension_origin_and_full_len(app, Some(v), o_idx);
            let line_offset = stored_offset(app, ortho_name, best_line_idx, l_count);
            let global_ortho = (origin_ortho + line_offset).min(full_ortho_len.saturating_sub(1));
            let ortho_str = format_dimension_coord(
                meta,
                Some(v),
                Some(&app.plotted_store_target_input),
                ortho_name,
                global_ortho,
                full_ortho_len,
                None,
            );
            entries.insert(0, ortho_str);
            used_dims.insert(o_idx);
        } else {
            entries.insert(0, HoverField::index_of("series", best_line_idx, l_count));
        }
    } else {
        entries.insert(0, HoverField::index_of("series", best_line_idx, l_count));
    }
}
