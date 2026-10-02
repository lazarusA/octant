//! Main hero landing page assembly and layout.

use std::time::Duration;
use web_time::Instant;

use super::chips::sample_slash_chips_row;
use super::feedback::{
    header_title, render_drag_hover_cue, render_idle_hint, render_status_pill,
    render_warning_banner,
};
use super::intake::intake_row;
use super::style::{gap, vspace};
use super::widget::draw_octant_widget;
use crate::app::OctantApp;

/// Render the clean, centered Hero Landing page.
pub fn show_hero_landing(app: &mut OctantApp, ui: &mut egui::Ui) {
    let now = Instant::now();
    let (filled, extra_rot, extra_scale) = app.hero_state.update_animation(now);

    // Keep animating smoothly at 60 FPS while wandering or loading.
    ui.ctx().request_repaint_after(Duration::from_millis(16));

    let is_drag_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    if is_drag_hovering {
        ui.ctx().request_repaint();
    }

    // Check warning state from drop handler
    let warning_id = egui::Id::new("drop_zone_warning_state");
    let active_warning: Option<crate::ui::drop_zone::DropZoneWarningState> =
        ui.ctx().data(|d| d.get_temp(warning_id));
    let is_warning_active = active_warning
        .as_ref()
        .is_some_and(|w| w.triggered_at.elapsed() < Duration::from_millis(3500));

    if is_warning_active {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }

    // Main centered composition with procedural cube, title, and intake
    let available_w = ui.available_width();
    let available_h = ui.available_height();
    let octant_size = (available_h * 0.22)
        .clamp(80.0, 136.0)
        .min((available_w * 0.40).max(80.0));
    let top_spacing = (available_h * 0.14).clamp(16.0, 80.0);
    let space = |base: f32| vspace(base, available_h);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(available_w);
            ui.set_width(available_w);

            ui.vertical_centered(|ui| {
                ui.set_min_width(available_w);
                ui.set_width(available_w);

                ui.add_space(top_spacing);

                // Centered 3D Octant procedural widget (interactive click to hop)
                let octant_resp =
                    draw_octant_widget(ui, octant_size, filled, extra_rot, extra_scale);
                if octant_resp.on_hover_text("Click to hop octant").clicked() {
                    app.hero_state.start_hop(Duration::from_millis(350));
                }

                ui.add_space(space(gap::CUBE_TITLE));
                header_title(ui);

                ui.add_space(space(gap::TITLE_INTAKE));
                intake_row(ui, app);

                // Helper caption under the input; drag / warning feedback
                // temporarily takes its place.
                ui.add_space(gap::INTAKE_HINT);
                if is_warning_active {
                    render_warning_banner(ui);
                } else if is_drag_hovering {
                    render_drag_hover_cue(ui);
                } else {
                    render_idle_hint(ui);
                }

                ui.add_space(space(gap::HINT_CHIPS));
                sample_slash_chips_row(ui, app);

                if app.is_loading || app.hero_state.loading {
                    let label = if !app.hero_state.source_label.is_empty() {
                        format!("loading: {}", app.hero_state.source_label)
                    } else {
                        "loading...".to_string()
                    };
                    ui.add_space(space(gap::CHIPS_STATUS));
                    render_status_pill(
                        ui,
                        crate::ui::icons::Icon::Hourglass,
                        crate::ui::icons::IconTone::Muted.color(ui.visuals()),
                        &label,
                        ui.visuals().text_color(),
                    );
                } else if app.hero_state.loaded && !app.hero_state.source_label.is_empty() {
                    let label = format!("loaded: {}", app.hero_state.source_label);
                    ui.add_space(space(gap::CHIPS_STATUS));
                    render_status_pill(
                        ui,
                        crate::ui::icons::Icon::Check,
                        crate::ui::icons::IconTone::Accent.color(ui.visuals()),
                        &label,
                        ui.visuals().text_color(),
                    );
                }

                ui.add_space(top_spacing);
            });
        });
}
