use crate::app::OctantApp;
use crate::plots::PlotType;

pub fn show_plot_type_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let is_3d_available = if let Some(meta) = &app.active_dataset_metadata {
        if let Some(v) = meta.variables.get(app.selected_variable_idx) {
            v.shape.len() >= 3 || v.dimension_names.len() >= 3
        } else {
            false
        }
    } else {
        false
    };

    let current_label = match app.active_plot_type {
        PlotType::Heatmap => "🌐 Plot: 2D Plane",
        PlotType::Sphere => "🌐 Plot: 3D Globe",
        PlotType::Surface | PlotType::Block => "🌐 Plot: 3D Surface / Blocks",
        PlotType::Volume => "🌐 Plot: 3D Volume",
        PlotType::PointCloud => "🌐 Plot: 3D Point Cloud",
    };

    ui.menu_button(current_label, |ui| {
        ui.set_min_width(210.0);

        ui.label(
            egui::RichText::new("Select Visualization Projection")
                .small()
                .weak(),
        );
        ui.separator();

        let options = [
            (PlotType::Heatmap, "🗺️ 2D Plane (Flatmap)", true),
            (PlotType::Sphere, "🌍 3D Globe (Sphere)", true),
            (PlotType::Surface, "⛰️ 3D Surface / Blocks", true),
            (PlotType::Volume, "☁️ 3D Volume Raycasting", is_3d_available),
            (PlotType::PointCloud, "✨ 3D Point Cloud", is_3d_available),
        ];

        for (plot_type, label, enabled) in options {
            let is_selected = app.active_plot_type == plot_type;
            if enabled {
                if ui.selectable_label(is_selected, label).clicked() {
                    app.active_plot_type = plot_type;
                    ui.close_menu();
                }
            } else {
                ui.add_enabled(
                    false,
                    egui::SelectableLabel::new(false, format!("{} (Requires 3D Data)", label)),
                );
            }
        }
    });
}
