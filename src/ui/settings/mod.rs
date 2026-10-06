mod clipping;
mod coastline;
pub(crate) mod composite;
mod export;
mod opacity;
mod plot_2d;

mod plot_3d;
mod plot_options;
mod resampling;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

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

    let mut should_close = false;
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
                    let header_id = ui.make_persistent_id("settings_panel_header");
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        header_id,
                        true,
                    )
                    .show_header(ui, |ui| {
                        should_close =
                            ui.panel_header(Icon::Settings, "Settings", "Close Settings");
                    })
                    .body(|ui| show_settings_body(app, ui));
                });
        });

    if should_close {
        app.show_settings_panel = false;
    }

    // Store width for next frame so Variable Controls can position to the right.
    app.settings_overlay_width = area_resp.response.rect.width();
}

/// Plot options, clipping bounds and export preferences, separated by rules.
fn show_settings_body(app: &mut OctantApp, ui: &mut egui::Ui) {
    plot_options::show_plot_options(app, ui);
    ui.separator();
    clipping::show_clipping_bounds(app, ui);
    ui.separator();
    export::show_export_preferences(app, ui);
}
