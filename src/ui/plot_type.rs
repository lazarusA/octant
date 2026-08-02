use crate::app::OctantApp;
use crate::plots::PlotType;

pub fn show_plot_type_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let current_label = match app.active_plot_type {
        PlotType::Heatmap => "🌐 Plot: 2D Plane",
        PlotType::Sphere => "🌐 Plot: 3D Globe",
        PlotType::Surface | PlotType::Block => "🌐 Plot: 3D Surface / Blocks",
        PlotType::Volume => "🌐 Plot: 3D Volume",
        PlotType::PointCloud => "🌐 Plot: 3D Point Cloud",
    };

    ui.menu_button(current_label, |ui| {
        ui.set_min_width(190.0);

        ui.label(egui::RichText::new("Select Visualization Projection").small().weak());
        ui.separator();

        let options = [
            (PlotType::Heatmap, "🗺️ 2D Plane (Flatmap)", true),
            (PlotType::Sphere, "🌍 3D Globe (Sphere)", true),
            (PlotType::Surface, "⛰️ 3D Surface / Blocks", true),
            (PlotType::Volume, "☁️ 3D Volume Raycasting", false),
            (PlotType::PointCloud, "✨ 3D Point Cloud", false),
        ];

        for (plot_type, label, enabled) in options {
            let is_selected = app.active_plot_type == plot_type;
            if enabled {
                if ui.selectable_label(is_selected, label).clicked() {
                    app.active_plot_type = plot_type;
                    ui.close_menu();
                }
            } else {
                ui.add_enabled(false, egui::SelectableLabel::new(false, format!("{} (Soon)", label)));
            }
        }
    });

    if app.active_plot_type == PlotType::Sphere || app.active_plot_type == PlotType::Surface {
        ui.separator();

        let pause_label = if app.sphere_auto_rotate { "⏸ Pause" } else { "▶ Rotate" };
        if ui.button(egui::RichText::new(pause_label).small()).clicked() {
            app.sphere_auto_rotate = !app.sphere_auto_rotate;
        }

        if ui.button(egui::RichText::new("↺ Reset View").small()).clicked() {
            app.sphere_zoom = 2.5;
            app.sphere_rotation_x = 0.4;
            app.sphere_rotation_y = 0.0;
        }
    }

    if app.active_plot_type == PlotType::Sphere {
        ui.separator();

        let style_label = match app.sphere_mode {
            0 => "🌍 Smooth Globe",
            1 => "🌋 Smooth Terrain",
            2 => "📐 Flat Steps",
            _ => "🧱 3D Radial Legos",
        };
        if ui.button(egui::RichText::new(style_label).small()).clicked() {
            app.sphere_mode = (app.sphere_mode + 1) % 4;
        }

        if app.sphere_mode > 0 {
            ui.add(
                egui::Slider::new(&mut app.sphere_displacement_strength, 0.0..=5.0)
                    .text("🌋 Height"),
            );
        }
    }

    if app.active_plot_type == PlotType::Surface {
        ui.separator();

        let style_label = match app.surface_mode {
            0 => "🌊 Smooth Terrain",
            1 => "📐 Flat Steps",
            _ => "🧱 3D Lego Cubes",
        };
        if ui.button(egui::RichText::new(style_label).small()).clicked() {
            app.surface_mode = (app.surface_mode + 1) % 3;
        }

        ui.add(
            egui::Slider::new(&mut app.surface_displacement_strength, 0.0..=5.0)
                .text("⛰️ Height"),
        );
    }
}
