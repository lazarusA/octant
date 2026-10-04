//! Interactive hover tooltips, data-point cards with leader lines, and 3D raycasting.

pub mod camera;
pub mod card;
pub mod enrich;
pub mod entries;
pub mod entries_1d;
pub mod entries_2d;
pub mod entries_3d;
pub mod field;
pub mod format;
pub mod hit;
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
pub use hit::{resolve_hit_coordinates, resolve_target_screen_pos};
pub use raycast_sphere::{get_normalized_radial_dr, raycast_sphere, sphere_target_pos};
pub use raycast_surface::{get_normalized_surface_height, raycast_surface, surface_target_pos};
pub use raycast_volume::{VolumeSampler, volume_target_pos};
pub use sample_1d::{draw_line_guidelines_and_reticle, sample_line_series, screen_to_norm_1d};
pub use sample_2d::Transform2D;

use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::utils::colormap::evaluate_color_cpu;
use card::{Anchoring, HoverCard, HoverValue};
use egui::{Color32, Rect};
use entries::{resolve_cell_value_and_dim_entries, resolve_variable_units};

/// Renders the hover card and its leader line for the hovered data point.
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

    let canvas_plot_type = app.effective_canvas_plot_type();
    if canvas_plot_type == PlotType::Line {
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

    let is_rgb = app.rgb_composite_mode;
    let color_params = app.get_color_params();
    let pixel_color = if canvas_plot_type == PlotType::Line && app.line_use_custom_color {
        Color32::from_rgba_unmultiplied(
            (app.line_color[0] * 255.0).clamp(0.0, 255.0) as u8,
            (app.line_color[1] * 255.0).clamp(0.0, 255.0) as u8,
            (app.line_color[2] * 255.0).clamp(0.0, 255.0) as u8,
            (app.line_color[3] * 255.0).clamp(0.0, 255.0) as u8,
        )
    } else {
        evaluate_color_cpu(raw_val, &color_params)
    };

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
            value: HoverValue::from_raw(raw_val, is_rgb),
            units: units_str,
            swatch: pixel_color,
            fields: &dim_entries,
        },
    );
}
