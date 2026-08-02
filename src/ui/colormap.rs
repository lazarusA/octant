use crate::app::OctantApp;

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🎨 Colormap", |ui| {
        ui.set_min_width(180.0);
        ui.label(egui::RichText::new("GPU Colormap Routine").strong());
        ui.separator();

        let colormaps = [
            (0, "Viridis (Thermal)"),
            (1, "Plasma (Spectral)"),
            (2, "Inferno (Radiance)"),
            (3, "Magma (Density)"),
        ];

        for (id, name) in colormaps {
            if ui.selectable_label(app.active_colormap == id, name).clicked() {
                app.active_colormap = id;
                ui.close_menu();
            }
        }
    });
}
