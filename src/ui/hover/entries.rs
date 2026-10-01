use crate::app::OctantApp;
use crate::data::{DatasetMetadata, MatrixData, VariableInfo};
use crate::plots::PlotType;
use crate::ui::hover::entries_1d::resolve_line_plot_entries;
use crate::ui::hover::entries_2d::resolve_2d_plot_entries;
use crate::ui::hover::entries_3d::resolve_3d_dim_entries;
use crate::ui::hover::raycast_volume::VolumeSampler;

pub(crate) fn resolve_variable_units(var: Option<&VariableInfo>) -> String {
    var.and_then(|v| {
        v.units
            .as_deref()
            .or(v.attributes.get("units").map(|s| s.as_str()))
    })
    .map(|u| {
        let clean = u.trim();
        if clean.is_empty() || clean == "1" || clean == "none" || clean == "dimensionless" {
            String::new()
        } else {
            format!("\u{00A0}{}", clean)
        }
    })
    .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_cell_value_and_dim_entries(
    app: &OctantApp,
    matrix: &MatrixData,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    sampler: &VolumeSampler,
    norm_x: f32,
    norm_y: f32,
    geo_coords: Option<(f32, f32)>,
    point_3d_hit: Option<(usize, usize, usize, f32)>,
) -> (f32, Vec<String>, usize, usize) {
    if app.effective_canvas_plot_type() == PlotType::Line {
        resolve_line_plot_entries(app, meta, var, norm_x, norm_y)
    } else if let Some((hit_x, hit_y, hit_z, hit_val)) = point_3d_hit {
        let entries = resolve_3d_dim_entries(app, meta, var, sampler, hit_x, hit_y, hit_z);
        (hit_val, entries, hit_x, hit_y)
    } else {
        resolve_2d_plot_entries(app, matrix, meta, var, norm_x, norm_y, geo_coords)
    }
}
