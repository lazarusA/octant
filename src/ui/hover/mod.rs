//! Interactive hover tooltips, data-point cards with leader lines, and 3D raycasting.

pub mod camera;
pub mod card;
pub mod composite;
mod composite_bands;
pub mod enrich;
pub mod entries;
pub mod entries_1d;
pub mod entries_2d;
pub mod entries_3d;
pub mod field;
pub mod format;
pub mod hit;
pub mod overlays;
pub mod raycast_sphere;
pub mod raycast_surface;
pub mod raycast_volume;
pub mod sample_1d;
pub mod sample_2d;

pub use camera::{Camera3D, Ray3D, intersect_aabb};
pub use enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
};
pub use field::HoverField;
pub use format::format_dimension_coord;
pub use hit::{HitResult, resolve_hit_coordinates, resolve_target_screen_pos};
pub use raycast_sphere::{get_normalized_radial_dr, raycast_sphere, sphere_target_pos};
pub use raycast_surface::{get_normalized_surface_height, raycast_surface, surface_target_pos};
pub use raycast_volume::{VolumeSampler, volume_target_pos};
pub use sample_1d::{draw_line_guidelines_and_reticle, sample_line_series, screen_to_norm_1d};
pub use sample_2d::Transform2D;

#[cfg(test)]
mod composite_kind_tests;
#[cfg(test)]
mod composite_tests;
#[cfg(test)]
mod flip_tests;
#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::app::overlays::MAX_OVERLAYS;
use crate::data::{DatasetMetadata, MatrixData, VariableInfo};
use crate::plots::PlotType;
use crate::utils::colormap::evaluate_color_cpu;
use card::{Anchoring, HoverCard, HoverValue, LayerValue};
use egui::{Color32, Pos2, Rect};
use entries::{resolve_cell_value_and_dim_entries, resolve_variable_units};

/// Renders the hover card and its leader line for the hovered data point.
pub fn show_hover_tooltip(
    app: &OctantApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    response: &egui::Response,
    rect: Rect,
) {
    let Some(hover_pos) = response.hover_pos() else {
        return;
    };
    if !app.show_hover_card {
        return;
    }

    let matrix = app
        .layers
        .base
        .data
        .matrix
        .as_ref()
        .filter(|m| m.width > 0 && m.height > 0 && !m.values.is_empty());
    let sampler = VolumeSampler::from_app(app, matrix);
    if matrix.is_none() && sampler.is_none() {
        return;
    }

    let Some((camera, hit)) = resolve_hit(app, matrix, sampler.as_ref(), rect, hover_pos) else {
        return;
    };

    let (meta, var, var_name, units_str) = resolve_hover_target_info(app);
    let (raw_val, mut dim_entries, px, py) = resolve_cell_value_and_dim_entries(
        app,
        matrix,
        meta,
        var,
        sampler.as_ref(),
        hit.norm_x,
        hit.norm_y,
        hit.geo_coords,
        hit.point_3d,
    );

    let canvas_plot_type = app.effective_canvas_plot_type();
    let flat = hit.point_3d.is_none() && canvas_plot_type != PlotType::Line;
    let pixel = flat.then_some((px, py));
    let composite =
        composite::composite_rows(app, ctx, meta, var, units_str, pixel, &mut dim_entries);
    if canvas_plot_type == PlotType::Line {
        draw_line_guidelines_and_reticle(app, ctx, ui, rect, px, raw_val);
    }

    let transform_2d = Transform2D::from_app(app, rect);
    let target_pos = resolve_target_screen_pos(
        app,
        matrix,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        px,
        py,
        raw_val,
        hit.point_3d,
    );

    paint_hover_card(
        app,
        ctx,
        rect,
        hover_pos,
        target_pos,
        canvas_plot_type,
        resolve_hover_color(app, raw_val, py),
        HoverValue::from_raw(raw_val, composite),
        var_name,
        var,
        units_str,
        pixel,
        &dim_entries,
    );
}

fn resolve_hit(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    sampler: Option<&VolumeSampler>,
    rect: Rect,
    hover_pos: Pos2,
) -> Option<(Camera3D, HitResult)> {
    let camera = Camera3D::from_app(app, rect);
    let transform_2d = Transform2D::from_app(app, rect);
    let hit = resolve_hit_coordinates(
        app,
        matrix,
        &camera,
        sampler,
        &transform_2d,
        rect,
        hover_pos,
    );
    if !hit.is_valid {
        return None;
    }
    Some((camera, hit))
}

fn resolve_hover_target_info(
    app: &OctantApp,
) -> (Option<&DatasetMetadata>, Option<&VariableInfo>, &str, &str) {
    let meta = app
        .plotted()
        .metadata
        .as_ref()
        .or(app.selected.metadata.as_ref());

    let var = meta.and_then(|m| {
        m.variables
            .get(app.plotted().variable_idx)
            .or_else(|| m.variables.get(app.selected.variable_idx))
            .or_else(|| m.variables.first())
    });

    let default_name = app
        .layers
        .base
        .data
        .volume
        .as_ref()
        .map(|v| v.dataset_name.as_str())
        .unwrap_or("variable");
    let var_name = var.map(|v| v.name.as_str()).unwrap_or(default_name);
    let units_str = resolve_variable_units(var);

    (meta, var, var_name, units_str)
}

/// The card's swatch: the color `raw_val` is drawn in, or on a line plot
/// colored by series, the color of series line `line`.
fn resolve_hover_color(app: &OctantApp, raw_val: f32, line: usize) -> Color32 {
    if app.line_series_colored() {
        app.line_series_color(line)
    } else if app.line_custom_colored() {
        let [r, g, b, a] = app.line_color.map(|c| (c * 255.0).clamp(0.0, 255.0) as u8);
        Color32::from_rgba_unmultiplied(r, g, b, a)
    } else {
        let color_params = app.get_color_params(&app.layers.base);
        evaluate_color_cpu(raw_val, &color_params)
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_hover_card(
    app: &OctantApp,
    ctx: &egui::Context,
    rect: Rect,
    hover_pos: Pos2,
    target_pos: Option<Pos2>,
    canvas_plot_type: PlotType,
    swatch: Color32,
    value: HoverValue,
    var_name: &str,
    var: Option<&VariableInfo>,
    units: &str,
    pixel: Option<(usize, usize)>,
    fields: &[HoverField],
) {
    let mut rows = [LayerValue::EMPTY; MAX_OVERLAYS];
    let layers = overlays::hover_rows(app, canvas_plot_type, pixel, &mut rows);
    let title = HoverCard::title_for(var_name, var.and_then(|v| v.long_name.as_deref()));
    let anchoring = if canvas_plot_type == PlotType::Line {
        Anchoring::FollowPointer
    } else {
        Anchoring::Connected
    };
    card::show_card(
        ctx,
        rect,
        hover_pos,
        target_pos,
        anchoring,
        &HoverCard {
            title,
            value,
            units,
            swatch,
            layers,
            fields,
        },
    );
}
