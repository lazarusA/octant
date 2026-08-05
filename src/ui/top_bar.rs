use super::{colormap, plot_type, status, store, variables};
use crate::app::OctantApp;

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::Panel::top("octant_top_bar")
        .exact_size(34.0)
        .show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                // Octant Brand Header
                ui.label(egui::RichText::new("📐 Octant").strong().heading());
                ui.separator();

                // Dropdown menus: Store, Catalog, Variables, Colormap, Plot Type, Settings
                store::show_store_menu(app, ui);

                if ui
                    .button(egui::RichText::new("📚 Catalog").strong())
                    .clicked()
                {
                    app.show_catalog_window = true;
                }

                variables::show_variables_menu(app, ui);
                colormap::show_colormap_menu(app, ui);
                plot_type::show_plot_type_menu(app, ui);

                if ui
                    .button(egui::RichText::new("⚙️ Settings").strong())
                    .clicked()
                {
                    app.show_settings_panel = !app.show_settings_panel;
                }

                if ui
                    .button(egui::RichText::new("🎛️ Controls").strong())
                    .on_hover_text("Toggle Variable Controls Panel")
                    .clicked()
                {
                    app.show_variable_controls = !app.show_variable_controls;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_label = if app.theme_preference == egui::ThemePreference::Dark {
                        "☀ Light"
                    } else {
                        "🌙 Dark"
                    };
                    if ui
                        .button(egui::RichText::new(theme_label).small())
                        .on_hover_text("Toggle light and dark theme")
                        .clicked()
                    {
                        app.theme_preference =
                            if app.theme_preference == egui::ThemePreference::Dark {
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
