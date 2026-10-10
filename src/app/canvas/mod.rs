//! Canvas rendering engine: viewport projection, user interaction, axis rendering, and overlays.

mod axes;
mod interactions;
mod overlays;
mod viewport;

pub use axes::draw_canvas_axes;
pub use interactions::handle_canvas_interactions;
pub use overlays::draw_canvas_overlays;
#[allow(unused_imports)]
pub use viewport::ViewportUniforms;
pub use viewport::compute_viewport_uniforms;

use crate::app::OctantApp;

/// Renders the central plotting canvas or the hero landing view when inactive.
pub fn render_canvas(app: &mut OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    let ctx = ui.ctx().clone();

    // Check screenshot / export requests requiring the canvas rect
    if let Some(ref mut req) = app.pending_export
        && req.canvas_rect_in_points == egui::Rect::NOTHING
    {
        req.canvas_rect_in_points = canvas_rect;
        req.pixels_per_point = ctx.pixels_per_point();
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        ctx.request_repaint();
    }

    let is_hero_active = (app.layers.base.data.matrix.is_none()
        && app.layers.base.data.volume.is_none())
        || app.layout.show_hero;

    // When no dataset is plotted or hero landing view is active, render the clean hero page
    if is_hero_active {
        let canvas_bg = ui.visuals().panel_fill;
        ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);
        crate::ui::hero::show_hero_landing(app, ui);
        return;
    }

    let response = ui.allocate_rect(canvas_rect, egui::Sense::drag());

    let canvas_bg = ui.style().visuals.panel_fill;
    ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);

    let canvas_plot_type = app.effective_canvas_plot_type();

    // Handle Zoom, Pan, Orbit, and Reset interactions
    handle_canvas_interactions(app, ui, &response, canvas_rect, canvas_plot_type);

    // Compute screen-space transformed plot rect and GPU pan/zoom uniforms
    let uniforms = compute_viewport_uniforms(app, canvas_rect, canvas_plot_type);

    // Volume planes changed since the last frame reach the shown renderer
    app.flush_volume_uploads();

    // Dispatch active plot GPU rendering callback
    app.paint_active_plot(
        ui,
        canvas_rect,
        uniforms.plot_rect,
        uniforms.gpu_pan,
        uniforms.gpu_zoom,
        uniforms.gpu_aspect_scale,
    );

    // Draw Dynamic Plot Axis Lines, Ticks, and Axis Titles
    draw_canvas_axes(app, ui, canvas_rect, uniforms.plot_rect, canvas_plot_type);

    // Render high-performance Hover Pixel Info Tooltip & Canvas Reticle (suppressed during export capture)
    if app.pending_export.is_none() {
        crate::ui::hover_tooltip::show_hover_tooltip(app, &ctx, ui, &response, canvas_rect);
    }

    // Render interactive ROI crop box, capture flash, and drag cue
    draw_canvas_overlays(app, ui, canvas_rect);
}
