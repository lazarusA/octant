mod clipping;
mod coastline;
pub(crate) mod composite;
mod export;
mod plot_2d;

mod plot_3d;
mod plot_options;
mod resampling;

use crate::app::OctantApp;

/// Anchored to the left edge of the canvas area, just below the top bar.
/// Stores its own width so Variable Controls can position to the right without overlap.
pub fn show_settings_window(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_settings_panel {
        app.settings_overlay_width = 0.0;
        return;
    }

    let x_offset = if app.show_variables_overlay && app.variables_overlay_width > 0.0 {
        app.variables_overlay_width + 16.0
    } else {
        8.0
    };

    let area_resp = egui::Area::new(egui::Id::new("octant_settings_area"))
        .fixed_pos(egui::pos2(
            canvas_rect.left() + x_offset,
            canvas_rect.top() + 8.0,
        ))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.set_max_width(280.0);
                    egui::CollapsingHeader::new("Settings")
                        .default_open(true)
                        .show(ui, |ui| {
                            plot_options::show_plot_options(app, ui);
                            ui.separator();
                            clipping::show_clipping_bounds(app, ui);
                            ui.separator();
                            export::show_export_preferences(app, ui);
                        });
                });
        });

    // Store width for next frame so Variable Controls can position to the right.
    app.settings_overlay_width = area_resp.response.rect.width();
}
