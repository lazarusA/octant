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
mod hit_cache;
pub mod labels;
mod march;
pub mod overlays;
pub mod raycast_sphere;
pub mod raycast_surface;
pub mod raycast_volume;
pub mod sample_1d;
pub mod sample_2d;
mod smooth;

pub use camera::{Camera3D, Ray3D, intersect_aabb};
pub use enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
};
pub use field::HoverField;
pub use format::format_dimension_coord;
pub use hit::{HitResult, resolve_hit_coordinates, resolve_target_screen_pos};
pub use raycast_sphere::{raycast_sphere, sphere_target_pos};
pub use raycast_surface::{raycast_surface, surface_target_pos};
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
mod raycast_edge_tests;
#[cfg(test)]
mod raycast_tests;
#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::app::overlays::MAX_OVERLAYS;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::plots::PlotType;
use crate::utils::colormap::evaluate_color_cpu;
use card::{Anchoring, HoverCard, LayerValue};
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
    let Some(hover_pos) = response.hover_pos().filter(|_| app.layout.show_hover_card) else {
        return;
    };
    let base = &app.layers.base;
    let matrix =
        (base.data.matrix.as_ref()).filter(|m| m.width > 0 && m.height > 0 && !m.values.is_empty());
    let sampler = VolumeSampler::from_app(app, matrix);
    let sampler = sampler.as_ref();
    let hit = (matrix.is_some() || sampler.is_some())
        .then(|| hit_cache::resolve_hit(app, ctx, matrix, sampler, rect, hover_pos));
    let Some((camera, hit)) = hit.flatten() else {
        return;
    };
    let (meta, var, var_name, units) = resolve_hover_target_info(app);
    let (raw_val, mut fields, px, py) =
        resolve_cell_value_and_dim_entries(app, matrix, meta, var, sampler, &hit);
    let point = hit.point_3d;
    let plot_type = app.effective_canvas_plot_type();
    let pixel = (point.is_none() && plot_type != PlotType::Line).then_some((px, py));
    // A label image reads its id, color and properties instead of a value.
    let composite = match base.labels() {
        Some(_) => None,
        None => composite::composite_rows(app, ctx, meta, var, units, pixel, &mut fields),
    };
    if plot_type == PlotType::Line {
        draw_line_guidelines_and_reticle(app, ctx, ui, rect, px, raw_val);
    }
    let transform = Transform2D::from_app(app, rect);
    let target =
        resolve_target_screen_pos(app, matrix, &camera, sampler, &transform, px, py, point);
    let swatch = || resolve_hover_color(app, raw_val, py);
    let (value, swatch, units) = labels::layer_reading(base, raw_val, composite, units, swatch);
    let long_name = var.and_then(|v| v.long_name.as_deref());
    let card = HoverCard {
        title: HoverCard::title_for(var_name, long_name),
        value,
        units,
        swatch,
        layers: &[],
        fields: &fields,
    };
    paint_hover_card(app, ctx, rect, (hover_pos, target), plot_type, pixel, card);
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
        .unwrap_or(crate::i18n::tr(crate::i18n::HoverKey::FallbackVariable));
    let var_name = var.map(|v| v.name.as_str()).unwrap_or(default_name);
    let units_str = resolve_variable_units(var);

    (meta, var, var_name, units_str)
}

/// The card's swatch: the color `raw_val` is drawn in, or on a line plot
/// colored by series, the color of series line `line` (a missing value takes
/// the NaN color, as no line is under it).
fn resolve_hover_color(app: &OctantApp, raw_val: f32, line: usize) -> Color32 {
    if app.line_series_colored() && raw_val.is_finite() {
        app.line_series_color(line)
    } else if app.line_custom_colored() {
        crate::utils::colormap::eval::rgba_to_color32(app.plot_configs.line.line_color)
    } else {
        let color_params = app.get_color_params(&app.layers.base);
        evaluate_color_cpu(raw_val, &color_params)
    }
}

/// Shows `card` with the overlays' rows at `pixel`, its leader running
/// from the pointer to the hovered data point's screen position.
fn paint_hover_card(
    app: &OctantApp,
    ctx: &egui::Context,
    rect: Rect,
    (hover_pos, target_pos): (Pos2, Option<Pos2>),
    plot_type: PlotType,
    pixel: Option<(usize, usize)>,
    card: HoverCard,
) {
    let mut rows = [LayerValue::EMPTY; MAX_OVERLAYS];
    let layers = overlays::hover_rows(app, plot_type, pixel, &mut rows);
    let anchoring = if plot_type == PlotType::Line {
        Anchoring::FollowPointer
    } else {
        Anchoring::Connected
    };
    let card = HoverCard { layers, ..card };
    card::show_card(ctx, rect, hover_pos, target_pos, anchoring, &card);
}
