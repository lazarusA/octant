//! Dataset intake bar and input field controls.

use super::style::{BODY_FONT, content_width, fit_text};
use crate::app::OctantApp;

/// Horizontal / vertical inner padding of the intake frame.
const MARGIN_X: i8 = 12;
const MARGIN_Y: i8 = 8;

/// Horizontal margin inside the text field itself (egui's default is 4).
const EDIT_MARGIN_X: i8 = 4;

/// Distance from the intake frame's outer left edge to its text (frame
/// stroke + frame padding + field margin), so captions below the bar can
/// line up with the input text.
pub const INTAKE_TEXT_INSET: f32 = 1.0 + MARGIN_X as f32 + EDIT_MARGIN_X as f32;

pub fn intake_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    // Outer width follows the shared hero gutter; subtract margins and stroke
    // so the frame itself never touches the window edge.
    let inner_w = content_width(ui.available_width()) - 2.0 * (f32::from(MARGIN_X) + stroke.width);

    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(stroke)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(MARGIN_X, MARGIN_Y))
        .show(ui, |ui| {
            ui.set_width(inner_w);
            ui.horizontal(|ui| {
                let has_input = !app.hero_state.input.trim().is_empty();
                let right_reserve = if has_input { 60.0 } else { 34.0 };
                let desired_w = (ui.available_width() - right_reserve).max(30.0);

                let hint_text = fit_text(
                    ui,
                    &[
                        "https://... or path (.zarr, .icechunk, .nc, .h5, .tiff, ...)",
                        "https://... or path (.zarr, .nc, .tiff, ...)",
                        "URL or path...",
                    ],
                    BODY_FONT,
                    desired_w,
                );

                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text(hint_text)
                    .font(egui::FontId::monospace(BODY_FONT))
                    .vertical_align(egui::Align::Center)
                    .min_size(egui::vec2(0.0, 22.0))
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::symmetric(EDIT_MARGIN_X, 2))
                    .desired_width(desired_w);
                let response = ui.add(edit);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if has_input
                    && crate::ui::icons::UiIconExt::close_button(ui, "Clear input").clicked()
                {
                    app.hero_state.input.clear();
                }

                // Procedural download / load icon button
                let btn_size = egui::vec2(26.0, 22.0);
                let (btn_rect, btn_response) =
                    ui.allocate_exact_size(btn_size, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    let btn_visuals = ui.style().interact(&btn_response);
                    ui.painter().rect(
                        btn_rect,
                        4.0,
                        btn_visuals.bg_fill,
                        btn_visuals.bg_stroke,
                        egui::StrokeKind::Inside,
                    );

                    let icon_rect = btn_rect.shrink(4.0);
                    crate::ui::icons::Icon::DropTray.paint(
                        ui.painter(),
                        icon_rect,
                        btn_visuals.fg_stroke.color,
                        ui.visuals().dark_mode,
                    );
                }

                let go_clicked = btn_response
                    .on_hover_text("Load Dataset & Open Variables")
                    .clicked();

                if enter_pressed || go_clicked {
                    let input_target = if !app.hero_state.input.trim().is_empty() {
                        app.hero_state.input.trim().to_string()
                    } else {
                        app.store_target_input.clone()
                    };

                    app.submit_or_activate_source(&input_target, None);
                }
            });
        });
}
