use crate::app::OctantApp;
use super::{colormap, status, store, variables};

pub fn show_top_bar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("octant_top_bar")
        .exact_height(34.0)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            egui::menu::bar(ui, |ui| {
                // Octant Brand Header
                ui.label(egui::RichText::new("📐 Octant").strong().heading());
                ui.separator();

                // Dropdown menus: Store, Catalog, Variables, Colormap
                store::show_store_menu(app, ui);

                if ui.button(egui::RichText::new("📚 Catalog").strong()).clicked() {
                    app.show_catalog_window = true;
                }

                variables::show_variables_menu(app, ui);
                colormap::show_colormap_menu(app, ui);

                ui.separator();
                let btn_text = if app.is_loading {
                    "⏳ Fetching..."
                } else {
                    "🔍 Fetch Store Metadata"
                };

                if ui.add_enabled(!app.is_loading, egui::Button::new(egui::RichText::new(btn_text).small())).clicked() {
                    app.inspect_active_store();
                }

                // Right status info
                status::show_status_bar(app, ui);
            });
        });
}
