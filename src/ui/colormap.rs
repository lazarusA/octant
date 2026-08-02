use crate::app::OctantApp;

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🎨 Colormap", |ui| {
        ui.set_min_width(170.0);

        ui.label(egui::RichText::new("Select Colormap Palette").small().weak());
        ui.separator();

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

        ui.separator();
        ui.checkbox(&mut app.show_colorbar, "📊 Show Colorbar Legend");
    });

    let bar_btn_label = if app.show_colorbar { "📊 Bar: On" } else { "📊 Bar: Off" };
    if ui.button(egui::RichText::new(bar_btn_label).small()).clicked() {
        app.show_colorbar = !app.show_colorbar;
    }
}
