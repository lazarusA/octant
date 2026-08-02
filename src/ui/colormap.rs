use crate::app::OctantApp;

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🎨 Colormap", |ui| {
        ui.set_min_width(170.0);

        let colormaps = [
            (0, "Viridis (Thermal)"),
            (1, "Plasma (Spectral)"),
            (2, "Inferno (Radiance)"),
            (3, "Magma (Density)"),
            (4, "Turbo (Rainbow)"),
            (5, "Coolwarm (Diverging)"),
            (6, "Cividis (Accessible)"),
        ];

        for (id, name) in colormaps {
            let is_active = app.active_colormap == id;
            let response = ui.selectable_label(is_active, name);

            if response.hovered() {
                app.preview_colormap = Some(id);
                ui.ctx().request_repaint();
            }

            if response.clicked() {
                app.active_colormap = id;
                app.preview_colormap = None;
                ui.close_menu();
            }
        }
    });
}
