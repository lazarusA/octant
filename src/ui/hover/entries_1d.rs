use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::enrich::{
    enrich_entries_with_animated_and_collapsed_dims, flipped_offset,
    get_dimension_origin_and_full_len, is_flipped, stored_offset,
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
    let (values, layout) = app.line_layout();
    let (prof_len, l_count) = (layout.profile_length, layout.line_count);

    let (sample_idx, best_line_idx, val) = sample_line_series(app, norm_x, norm_y, values, layout);
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
        Some(&app.plotted().store_target),
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
    let (field, dim) = series_field(app, meta, var, best_line_idx, l_count);
    entries.insert(0, field);
    used_dims.extend(dim);
}

/// The dimension series lines run across, placed in the plotted window: its
/// index and name, the window's first index and the dimension's length, and
/// whether blocks flipped it.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct SeriesDim<'a> {
    pub index: usize,
    pub name: &'a str,
    pub origin: usize,
    pub len: usize,
    pub flipped: bool,
}

/// The dimension the lines of `var`'s line plot run across: Y for rows, X
/// for columns, none for rays along Z.
pub(crate) fn series_dim<'a>(
    app: &OctantApp,
    var: Option<&'a VariableInfo>,
) -> Option<SeriesDim<'a>> {
    let v = var?;
    let (explicit_x, explicit_y, _) = v.resolve_spatial_dim_indices(app.effective_dim_config());
    let index = match app.line_profile_dim_idx {
        0 => explicit_y,
        1 => explicit_x,
        _ => None,
    }?;
    let name = v.dimension_names.get(index)?;
    let (origin, len) = get_dimension_origin_and_full_len(app, var, index);
    let flipped = is_flipped(app, name);
    Some(SeriesDim {
        index,
        name,
        origin,
        len,
        flipped,
    })
}

/// Where series line `line` of `count` sits: the coordinate of the dimension
/// the lines run across (with that dimension's index), else the line's index.
pub(crate) fn series_field(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    line: usize,
    count: usize,
) -> (HoverField, Option<usize>) {
    let Some((v, dim)) = var.zip(series_dim(app, var)) else {
        return (HoverField::index_of("series", line, count), None);
    };
    let offset = flipped_offset(dim.flipped, line, count);
    let global = (dim.origin + offset).min(dim.len.saturating_sub(1));
    let target = Some(app.plotted().store_target.as_str());
    let field = format_dimension_coord(meta, Some(v), target, dim.name, global, dim.len, None);
    (field, Some(dim.index))
}
