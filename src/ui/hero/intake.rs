//! Dataset intake bar and input field controls.

use crate::app::OctantApp;

pub fn intake_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let avail_w = ui.available_width();
    let intake_w = (avail_w - 24.0).clamp(180.0, 460.0);

    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(intake_w);
            ui.horizontal(|ui| {
                let has_input = !app.hero_state.input.trim().is_empty();
                let right_reserve = if has_input { 52.0 } else { 30.0 };

                let hint_text = if intake_w < 310.0 {
                    "URL or path..."
                } else if intake_w < 400.0 {
                    "https://... or path (.zarr, .nc, .tiff, ...)"
                } else {
                    "https://... or path (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
                };

                let desired_w = (ui.available_width() - right_reserve).max(30.0);
                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text(hint_text)
                    .font(egui::TextStyle::Monospace)
                    .frame(egui::Frame::NONE)
                    .desired_width(desired_w);
                let response = ui.add(edit);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if has_input {
                    let clear_size = egui::vec2(18.0, 18.0);
                    let (clear_rect, clear_resp) =
                        ui.allocate_exact_size(clear_size, egui::Sense::click());

                    if ui.is_rect_visible(clear_rect) {
                        let is_hovered = clear_resp.hovered();
                        let color = if is_hovered {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().weak_text_color().gamma_multiply(0.65)
                        };
                        crate::ui::icons::Icon::Cross.paint(
                            ui.painter(),
                            clear_rect.shrink(2.0),
                            color,
                            ui.visuals().dark_mode,
                        );
                    }

                    if clear_resp.on_hover_text("Clear input").clicked() {
                        app.hero_state.input.clear();
                    }
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
