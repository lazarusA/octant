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

                // Dropdown menus: Store, Catalog, Variables, Colormap, Plot Type
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
                    .button(egui::RichText::new("⚙ Settings").strong())
                    .clicked()
                {
                    app.show_settings_panel = !app.show_settings_panel;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let visibility_label = if app.show_bottom_bar {
                        "⬇ Hide Bottom"
                    } else {
                        "⬆ Show Bottom"
                    };
                    if ui
                        .button(egui::RichText::new(visibility_label).small())
                        .on_hover_text("Toggle bottom playback bar")
                        .clicked()
                    {
                        app.show_bottom_bar = !app.show_bottom_bar;
                    }

                    let settings_label = if app.show_settings_panel {
                        "⚙ Hide Settings"
                    } else {
                        "⚙ Show Settings"
                    };
                    if ui
                        .button(egui::RichText::new(settings_label).small())
                        .on_hover_text("Toggle settings panel")
                        .clicked()
                    {
                        app.show_settings_panel = !app.show_settings_panel;
                    }

                    let variables_label = if app.show_variable_controls {
                        "📊 Hide Variable"
                    } else {
                        "📊 Show Variable"
                    };
                    if ui
                        .button(egui::RichText::new(variables_label).small())
                        .on_hover_text("Toggle variable controls panel")
                        .clicked()
                    {
                        app.show_variable_controls = !app.show_variable_controls;
                    }

                    let left_label = if app.show_left_panel {
                        "◂ Hide Left"
                    } else {
                        "▸ Show Left"
                    };
                    if ui
                        .button(egui::RichText::new(left_label).small())
                        .on_hover_text("Toggle left store panel")
                        .clicked()
                    {
                        app.show_left_panel = !app.show_left_panel;
                    }

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
