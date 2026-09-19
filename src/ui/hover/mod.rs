//! Interactive hover tooltips, glassmorphic card overlays, and 3D raycasting.

pub mod callout;
pub mod camera;
pub mod enrich;
pub mod entries;
pub mod entries_1d;
pub mod entries_2d;
pub mod entries_3d;
pub mod format;
pub mod overlay;
pub mod raycast_sphere;
pub mod raycast_surface;
pub mod raycast_volume;
pub mod sample_1d;
pub mod sample_2d;

pub use callout::draw_leader_callout;
pub use camera::{Camera3D, Ray3D, intersect_aabb};
pub use enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
};
pub use format::format_dimension_coord;
pub use raycast_sphere::{get_normalized_radial_dr, raycast_sphere, sphere_target_pos};
pub use raycast_surface::{get_normalized_surface_height, raycast_surface, surface_target_pos};
pub use raycast_volume::{VolumeSampler, volume_target_pos};
pub use sample_1d::{draw_line_guidelines_and_reticle, sample_line_series, screen_to_norm_1d};
pub use sample_2d::Transform2D;

use crate::app::OctantApp;
use crate::plots::PlotType;
use egui::Rect;
use entries::{resolve_cell_value_and_dim_entries, resolve_variable_units};
use overlay::{draw_tooltip_card, resolve_hit_coordinates, resolve_target_screen_pos};

/// Renders the floating glassmorphic tooltip card and connecting leader lines for the hovered data point.
pub fn show_hover_tooltip(
    app: &OctantApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    response: &egui::Response,
    rect: Rect,
) {
    let hover_pos = match response.hover_pos() {
        Some(pos) => pos,
        None => return,
    };

    if !app.show_hover_card {
        return;
    }

    let matrix = match &app.matrix_data {
        Some(m) if m.width > 0 && m.height > 0 && !m.values.is_empty() => m,
        _ => return,
    };

    let camera = Camera3D::from_app(app, rect);
    let sampler = VolumeSampler::from_app(app, matrix);
    let transform_2d = Transform2D::from_app(app, rect, matrix);

    let (norm_x, norm_y, is_valid_hit, geo_coords, point_3d_hit) = resolve_hit_coordinates(
        app,
        matrix,
        &camera,
        &sampler,
        &transform_2d,
        rect,
        hover_pos,
    );

    if !is_valid_hit {
        return;
    }

    let meta = app
        .plotted_dataset_metadata
        .as_ref()
        .or(app.active_dataset_metadata.as_ref());

    let var = meta.and_then(|m| {
        m.variables
            .get(app.plotted_variable_idx)
            .or_else(|| m.variables.get(app.selected_variable_idx))
            .or_else(|| m.variables.first())
    });

    let var_name = var.map(|v| v.name.as_str()).unwrap_or("variable");
    let units_str = resolve_variable_units(var);

    let (raw_val, dim_entries, px, py) = resolve_cell_value_and_dim_entries(
        app,
        matrix,
        meta,
        var,
        &sampler,
        norm_x,
        norm_y,
        geo_coords,
        point_3d_hit,
    );

    if app.active_plot_type == PlotType::Line {
        draw_line_guidelines_and_reticle(app, ctx, ui, rect, px, raw_val);
    }

    let target_pos = resolve_target_screen_pos(
        app,
        matrix,
        &camera,
        &sampler,
        &transform_2d,
        px,
        py,
        raw_val,
        point_3d_hit,
    );

    let is_rgb = app.active_colormap == 1000 || app.rgb_composite_mode;
    draw_tooltip_card(
        ctx,
        ui,
        rect,
        hover_pos,
        target_pos,
        app.active_plot_type,
        var_name,
        raw_val,
        &units_str,
        &dim_entries,
        is_rgb,
    );
}
