use crate::app::OctantApp;
use super::{cache, colormap, status, store, variables};

pub fn show_top_bar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("octant_top_bar")
        .exact_height(34.0)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            egui::menu::bar(ui, |ui| {
                // Octant Brand Header
                ui.label(egui::RichText::new("📐 Octant").strong().heading());
                ui.separator();

                // Strict Menu Entries Order: Store, Variables, Colormap, Cache
                store::show_store_menu(app, ui);
                variables::show_variables_menu(app, ui);
                colormap::show_colormap_menu(app, ui);
                cache::show_cache_menu(app, ui);

                // Right status info
                status::show_status_bar(app, ui);
            });
        });
}
