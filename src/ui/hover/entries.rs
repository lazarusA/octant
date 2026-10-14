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
        resolve_2d_plot_entries(app, matrix, meta, var, norm_x, norm_y, hit.geo_coords)
    } else {
        (f32::NAN, Vec::new(), 0, 0)
    }
}
