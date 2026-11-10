use crate::app::OctantApp;
use crate::data::{DatasetMetadata, MatrixData, VariableInfo};
use crate::plots::PlotType;
use crate::ui::hover::entries_1d::resolve_line_plot_entries;
use crate::ui::hover::entries_2d::resolve_2d_plot_entries;
use crate::ui::hover::entries_3d::resolve_3d_dim_entries;
use crate::ui::hover::field::HoverField;
use crate::ui::hover::raycast_volume::VolumeSampler;
use crate::utils::units::is_dimensionless_unit;

/// Display units of the variable; empty for missing or dimensionless units.
pub(crate) fn resolve_variable_units(var: Option<&VariableInfo>) -> &str {
    var.and_then(|v| {
        v.units
            .as_deref()
            .or(v.attributes.get("units").map(|s| s.as_str()))
    })
    .map(str::trim)
    .filter(|u| !is_dimensionless_unit(u))
    .unwrap_or("")
}

/// The value, dimension rows and cell `(x, y)` at `hit`.
pub(crate) fn resolve_cell_value_and_dim_entries(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    sampler: Option<&VolumeSampler>,
    hit: &super::HitResult,
) -> (f32, Vec<HoverField>, usize, usize) {
    let (norm_x, norm_y) = (hit.norm_x, hit.norm_y);
    if app.effective_canvas_plot_type() == PlotType::Line {
        resolve_line_plot_entries(app, meta, var, norm_x, norm_y)
    } else if let Some((hit_x, hit_y, hit_z, hit_val)) = hit.point_3d {
        if let Some(sampler) = sampler {
            let entries = resolve_3d_dim_entries(app, meta, var, sampler, hit_x, hit_y, hit_z);
            (hit_val, entries, hit_x, hit_y)
        } else {
            (hit_val, Vec::new(), hit_x, hit_y)
        }
    } else if let Some(matrix) = matrix {
        resolve_2d_plot_entries(
            app,
            matrix,
            meta,
            var,
            (norm_x, norm_y),
            hit.cell,
            hit.geo_coords,
        )
    } else {
        (f32::NAN, Vec::new(), 0, 0)
    }
}

/// The X and Y dimensions of `v` a 2D or 3D hover reads (explicit roles or
/// names, else the last two dimensions), and its Z role.
pub(crate) fn plane_dims(app: &OctantApp, v: &VariableInfo) -> (usize, usize, Option<usize>) {
    let (x, y, z) = v.resolve_spatial_dim_indices(app.effective_dim_config());
    let rank = v.dimension_names.len();
    let x = x.unwrap_or(rank.saturating_sub(1));
    (x, y.unwrap_or(rank.saturating_sub(2)), z)
}

/// The Y and X coordinate rows of cell `(cx, cy)` of a `w` x `h` plane over
/// `v`'s dimensions `(x, y)`, named by those dimensions (so flipped ones map
/// back to stored rows), with the `geo` latitude and longitude when known.
pub(crate) fn plane_fields(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    v: &VariableInfo,
    (x, y): (usize, usize),
    (cx, cy, w, h): (usize, usize, usize, usize),
    geo: Option<(f32, f32)>,
) -> [HoverField; 2] {
    let store = Some(app.plotted().store_target.as_str());
    let row = |d: usize, fallback: &str, cell: usize, len: usize, geo: Option<f32>| {
        let name = v.dimension_names.get(d).map_or(fallback, String::as_str);
        let (origin, full) = super::enrich::get_dimension_origin_and_full_len(app, Some(v), d);
        let offset = super::enrich::stored_offset(app, name, cell, len);
        let at = (origin + offset).min(full.saturating_sub(1));
        super::format::format_dimension_coord(meta, Some(v), store, name, at, full, geo)
    };
    [
        row(y, "y", cy, h, geo.map(|g| g.0)),
        row(x, "x", cx, w, geo.map(|g| g.1)),
    ]
}
