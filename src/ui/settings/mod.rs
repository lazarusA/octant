mod clipping;
mod coastline;
pub(crate) mod composite;
mod export;
mod opacity;
#[cfg(test)]
mod panel_tests;
mod plot_2d;

mod plot_3d;
mod plot_options;
mod resampling;
mod scale;
mod support;
#[cfg(test)]
mod support_tests;
mod view;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};
use support::{PlotState, Support};

/// Shortest the scrolled settings body gets on a very short canvas.
const MIN_BODY_HEIGHT: f32 = 120.0;

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
                    .body(|ui| {
                        // Scroll the body rather than run past the canvas bottom.
                        let bottom_margin = 8.0 + f32::from(ui.style().spacing.menu_margin.bottom);
                        let max_height = (canvas_rect.bottom() - bottom_margin - ui.cursor().top())
                            .max(MIN_BODY_HEIGHT);
                        // The area lends its last-frame size; open up to the
                        // canvas so the body can grow when a section expands.
                        ui.set_max_height(max_height);
                        egui::ScrollArea::vertical()
                            .id_salt("settings_panel_scroll")
                            .max_height(max_height)
                            .show(ui, |ui| show_settings_body(app, ui));
                    });
                });
        });

    if should_close {
        app.show_settings_panel = false;
    }

    // Store width for next frame so Variable Controls can position to the right.
    app.settings_overlay_width = area_resp.response.rect.width();
}

/// The plot's own options, then color (with transparency), overlays,
/// resolution, view and export sections. Sections and settings the plot has no use for
/// are left out (`support`).
fn show_settings_body(app: &mut OctantApp, ui: &mut egui::Ui) {
    let plot_type = app.effective_canvas_plot_type();
    ui.label(egui::RichText::new(plot_type.display_name()).small().weak());
    plot_options::show_plot_options(app, ui, plot_type);
    // After the plot options, which may switch the volume algorithm or the
    // composite this frame.
    let support = PlotState::of(app).support();
    ui.add_space(4.0);
    ui.separator();
    egui::CollapsingHeader::new("Color")
        .id_salt("settings_color_section")
        .default_open(false)
        .show(ui, |ui| {
            clipping::show_color_settings(app, ui, &support);
            ui.add_space(4.0);
            ui.separator();
            opacity::show_transparency_settings(app, ui, &support);
        });
    if support.coastlines != Support::No {
        section(ui, "Overlays");
        coastline::show_coastline_controls(app, ui);
    }
    if support.aggregation != Support::No {
        section(ui, "Resolution");
        gated(ui, support.aggregation, |ui| {
            resampling::show_resampling_controls(app, ui);
        });
    }
    section(ui, "View");
    view::show_view_controls(app, ui, &support);
    ui.separator();
    export::show_export_preferences(app, ui);
}

/// A rule and a muted title opening a settings section.
fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.separator();
    ui.label(egui::RichText::new(title).small().weak());
}

/// Draws `add` when the setting applies, a muted note naming the setting that
/// overrides it, or nothing when the plot has no use for it.
fn gated(ui: &mut egui::Ui, support: Support, add: impl FnOnce(&mut egui::Ui)) {
    match support {
        Support::Yes => add(ui),
        Support::Overridden(reason) => note(ui, reason),
        Support::No => {}
    }
}

/// A muted, wrapped line explaining why settings are left out.
fn note(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(egui::RichText::new(text).small().weak()).wrap());
}
