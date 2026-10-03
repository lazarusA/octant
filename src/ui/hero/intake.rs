//! Dataset intake bar and input field controls.

use super::style::{BODY_FONT, content_width, fit_text};
use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// Horizontal / vertical inner padding of the intake frame.
const MARGIN_X: i8 = 12;
const MARGIN_Y: i8 = 8;

/// Horizontal margin inside the text field itself (egui's default is 4).
const EDIT_MARGIN_X: i8 = 4;

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

                // The field's own margin narrows the visible text area.
                let hint_text = intake_hint(ui, desired_w - 2.0 * f32::from(EDIT_MARGIN_X));

                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .id(super::focus::intake_id())
                    .hint_text(hint_text)
                    .font(egui::FontId::monospace(BODY_FONT))
                    .vertical_align(egui::Align::Center)
                    .min_size(egui::vec2(0.0, 22.0))
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::symmetric(EDIT_MARGIN_X, 2))
                    .desired_width(desired_w);
                let response = ui.add(edit);
                super::chip_nav::from_intake(ui.ctx(), &response);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if has_input && ui.close_button("Clear input").clicked() {
                    app.hero_state.input.clear();
                }

                if load_button(ui) || enter_pressed {
                    submit_intake(app);
                }
            });
        });
}

/// Longest input placeholder that fits in `text_w`.
fn intake_hint(ui: &egui::Ui, text_w: f32) -> &'static str {
    fit_text(
        ui,
        &[
            "https://... or path (.zarr, .icechunk, .nc, .h5, .tiff, ...)",
            "https://... or path (.zarr, .nc, .tiff, ...)",
            "URL or path...",
        ],
        BODY_FONT,
        text_w,
    )
}

/// Framed load button with a drop-tray glyph; returns `true` when clicked.
fn load_button(ui: &mut egui::Ui) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 22.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            rect,
            4.0,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        Icon::DropTray.paint(
            ui.painter(),
            rect.shrink(4.0),
            visuals.fg_stroke.color,
            ui.visuals().dark_mode,
        );
    }
    response
        .on_hover_text("Load Dataset & Open Variables")
        .clicked()
}

/// Load the typed source, or the current store target when the field is empty.
fn submit_intake(app: &mut OctantApp) {
    let typed = app.hero_state.input.trim();
    let target = if typed.is_empty() {
        app.store_target_input.clone()
    } else {
        typed.to_owned()
    };
    app.submit_or_activate_source(&target, None);
}
