mod clipping;
mod coastline;
pub(crate) mod composite;
mod composite_rgb;
mod export;
mod layers;
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
use crate::app::layers::LayerId;
use crate::ui::icons::Icon;
use crate::ui::panel_header::{self, PanelHeader};
use crate::ui::panel_layout::{self, Panel};
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

    let origin = panel_layout::origin(app, Panel::Settings, canvas_rect);
    let mut header = None;
    let area_resp = egui::Area::new(egui::Id::new("octant_settings_area"))
        .fixed_pos(origin)
        .constrain_to(canvas_rect)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| header = show_panel(app, ui, canvas_rect));
        });

    // Store width for next frame so Variable Controls can position to the right.
    let rect = area_resp.response.rect;
    app.settings_overlay_width = rect.width();
    if let Some(header) = header {
        panel_layout::apply_grip(app, Panel::Settings, header.grip, rect, canvas_rect);
        if header.close {
            app.show_settings_panel = false;
        }
    }
}

/// The collapsible "Settings" header over the scrolled body, which stops
/// short of the bottom of `canvas`; what the header's buttons asked for.
fn show_panel(app: &mut OctantApp, ui: &mut egui::Ui, canvas: egui::Rect) -> Option<PanelHeader> {
    ui.set_max_width(280.0);
    let header_id = ui.make_persistent_id("settings_panel_header");
    let mut header = None;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, true)
        .show_header(ui, |ui| {
            let title = panel_header::show(ui, Icon::Settings, "Settings", "Close Settings");
            header = Some(title);
        })
        .body(|ui| {
            // Scroll the body rather than run past the canvas bottom.
            let max_height = panel_layout::room_below(ui, canvas).max(MIN_BODY_HEIGHT);
            // The area lends its last-frame size; open up to the canvas so the
            // body can grow when a section expands.
            ui.set_max_height(max_height);
            egui::ScrollArea::vertical()
                .id_salt("settings_panel_scroll")
                .max_height(max_height)
                .show(ui, |ui| show_settings_body(app, ui));
        });
    header
}

/// The plot's own options, then the Layers menu (each layer's composite and
/// Color menu), overlays, resolution, view and export sections. Sections and settings the plot has no use for
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
    layers::show_layers_menu(app, ui);
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

/// Layer `id`'s Color menu: label, range, scale, NaN and clip colors, then
/// opacity and alpha curve, as far as the layer's plot honors them.
fn show_color_menu(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let support = PlotState::of_layer(app, id).support();
    clipping::show_color_settings(app, ui, id, &support);
    ui.add_space(4.0);
    ui.separator();
    opacity::show_transparency_settings(app, ui, id, &support);
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
