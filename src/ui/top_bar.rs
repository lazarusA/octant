use crate::app::OctantApp;
use super::{colormap, plot_type, status, store, variables};

pub fn show_top_bar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("octant_top_bar")
        .exact_height(34.0)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            egui::menu::bar(ui, |ui| {
                // Octant Brand Header
                ui.label(egui::RichText::new("📐 Octant").strong().heading());
                ui.separator();

                // Dropdown menus: Store, Catalog, Variables, Colormap, Plot Type
                store::show_store_menu(app, ui);

                if ui.button(egui::RichText::new("📚 Catalog").strong()).clicked() {
                    app.show_catalog_window = true;
                }

                variables::show_variables_menu(app, ui);
                colormap::show_colormap_menu(app, ui);
                plot_type::show_plot_type_menu(app, ui);

                if app.active_dataset_metadata.is_some() {
                    // Right status info and controls toggle on far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let panel_label = if app.show_right_panel { "🎛️ Controls ◀" } else { "🎛️ Controls ▶" };
                        if ui.button(egui::RichText::new(panel_label).strong()).on_hover_text("Toggle Right Variable Controls Panel").clicked() {
                            app.show_right_panel = !app.show_right_panel;
                        }
                        ui.separator();
                        status::show_status_bar(app, ui);
                    });
                } else {
                    status::show_status_bar(app, ui);
                }
            });
        });
}
