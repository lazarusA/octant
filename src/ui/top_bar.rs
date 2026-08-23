use super::{colormap, plot_type, status, store};
use crate::app::OctantApp;

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::Panel::top("octant_top_bar")
        .exact_size(34.0)
        .show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                // Octant Brand Header (clickable to toggle Hero / Landing view)
                let brand_resp = ui
                    .horizontal(|ui| {
                        let icon_resp = octant_icon(ui, 24.0);
                        let label_resp = ui.add(
                            egui::Label::new(egui::RichText::new("Octant").strong().heading())
                                .sense(egui::Sense::click()),
                        );
                        icon_resp | label_resp
                    })
                    .inner
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Toggle Hero / Landing View");

                if brand_resp.clicked() {
                    app.show_hero = !app.show_hero;
                }
                ui.separator();

                // Dropdown menus: Store, Variables, Dimensions, Plot, Settings
                store::show_store_menu(app, ui);

                if ui
                    .button(egui::RichText::new("📊 Variables").strong())
                    .clicked()
                {
                    app.show_variables_overlay = !app.show_variables_overlay;
                }

                if ui
                    .button(egui::RichText::new("🎛️ Dimensions").strong())
                    .on_hover_text("Toggle Variable Controls Panel")
                    .clicked()
                {
                    app.show_variable_controls = !app.show_variable_controls;
                }

                plot_type::show_plot_type_menu(app, ui);
                colormap::show_colormap_menu(app, ui);

                if ui
                    .button(egui::RichText::new("⚙️ Settings").strong())
                    .clicked()
                {
                    app.show_settings_panel = !app.show_settings_panel;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_dark = app.theme_preference == egui::ThemePreference::Dark;
                    let theme_label = if is_dark { "☀ Light" } else { "🌙 Dark" };
                    let theme_hover = if is_dark {
                        "Switch to Light mode"
                    } else {
                        "Switch to Dark mode"
                    };
                    if ui
                        .button(egui::RichText::new(theme_label))
                        .on_hover_text(theme_hover)
                        .clicked()
                    {
                        app.theme_preference = if is_dark {
                            egui::ThemePreference::Light
                        } else {
                            egui::ThemePreference::Dark
                        };
                        ui.ctx().set_theme(app.theme_preference);
                    }

                    ui.separator();
                    status::show_status_bar(app, ui);
                });
            });
        });
}

fn octant_icon(ui: &mut egui::Ui, size: f32) -> egui::Response {
    crate::ui::hero::draw_octant_widget(ui, size, [-1.0, -1.0, -1.0], 0.0, 1.0)
}
