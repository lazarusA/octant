//! Dataset intake bar and input field controls.

use super::style::{BODY_FONT, content_width, fit_text};
use crate::app::OctantApp;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};

/// Horizontal / vertical inner padding of the intake frame.
const MARGIN_X: i8 = 12;
const MARGIN_Y: i8 = 8;

/// Horizontal margin inside the text field itself (egui's default is 4).
const EDIT_MARGIN_X: i8 = 4;

/// How far the field's fill moves from the page toward the text color: at
/// rest, hovered and focused. A soft tint of the page instead of a border.
const FILL_REST: f32 = 0.06;
const FILL_HOVER: f32 = 0.09;
const FILL_FOCUS: f32 = 0.12;

pub fn intake_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    // Outer width follows the shared hero gutter; subtract the margins so the
    // frame itself never touches the window edge.
    let inner_w = content_width(ui.available_width()) - 2.0 * f32::from(MARGIN_X);

    let mut frame = egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(MARGIN_X, MARGIN_Y))
        .begin(ui);
    let focused = {
        let ui = &mut frame.content_ui;
        ui.set_width(inner_w);
        ui.horizontal(|ui| {
            let has_input = !app.layout.hero_state.input.trim().is_empty();
            let right_reserve = if has_input { 60.0 } else { 34.0 };
            let desired_w = (ui.available_width() - right_reserve).max(30.0);

            // The field's own margin narrows the visible text area.
            let hint_text = intake_hint(ui, desired_w - 2.0 * f32::from(EDIT_MARGIN_X));

            let edit = egui::TextEdit::singleline(&mut app.layout.hero_state.input)
                .id(super::focus::intake_id())
                .hint_text(hint_text)
                .font(egui::FontId::monospace(BODY_FONT))
                .align(egui::Align2::LEFT_CENTER)
                .min_size(egui::vec2(0.0, 22.0))
                .frame(egui::Frame::NONE)
                .margin(egui::Margin::symmetric(EDIT_MARGIN_X, 2))
                .desired_width(desired_w);
            let response = ui.add(edit);
            super::chip_nav::from_intake(ui.ctx(), &response);
            let focused = response.has_focus();

            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if has_input && ui.close_button(tr(DatasetKey::ClearInputHover)).clicked() {
                app.layout.hero_state.input.clear();
            }

            if load_button(ui) || enter_pressed {
                submit_intake(app);
            }
            focused
        })
        .inner
    };

    // Borderless pill whose fill deepens on hover and focus.
    let rect = frame.content_ui.min_rect() + frame.frame.total_margin();
    let step = if focused {
        FILL_FOCUS
    } else if ui.rect_contains_pointer(rect) {
        FILL_HOVER
    } else {
        FILL_REST
    };
    let visuals = ui.visuals();
    frame.frame.fill = visuals.panel_fill.lerp_to_gamma(visuals.text_color(), step);
    frame.frame.corner_radius = egui::CornerRadius::from(rect.height() * 0.5);
    frame.end(ui);
}

/// Longest input placeholder that fits in `text_w`.
fn intake_hint(ui: &egui::Ui, text_w: f32) -> &'static str {
    fit_text(
        ui,
        &[
            tr(DatasetKey::HeroInputHintLong),
            tr(DatasetKey::HeroInputHintMedium),
            tr(DatasetKey::HeroInputHintShort),
        ],
        BODY_FONT,
        text_w,
    )
}

/// Frameless load button with a drop-tray glyph; returns `true` when clicked.
fn load_button(ui: &mut egui::Ui) -> bool {
    let hover = tr(DatasetKey::LoadDatasetHover);
    ui.add(
        ToolbarButton::new(Icon::DropTray, hover)
            .compact(true)
            .icon_size(IconSize::Sm),
    )
    .clicked()
}

/// Load the typed source, or the current store target when the field is empty.
fn submit_intake(app: &mut OctantApp) {
    let typed = app.layout.hero_state.input.trim();
    if typed.eq_ignore_ascii_case("/catalog") || typed.eq_ignore_ascii_case("catalog") {
        app.layout.show_catalog_window = true;
        return;
    }
    let target = if typed.is_empty() {
        app.selected.store_target.clone()
    } else {
        typed.to_owned()
    };
    app.submit_or_activate_source(&target, None);
}
