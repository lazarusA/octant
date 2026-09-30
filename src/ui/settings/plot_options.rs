use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::settings::plot_2d::{show_heatmap_options, show_line_options};
use crate::ui::settings::plot_3d::{
    show_point_cloud_options, show_sphere_options, show_surface_options, show_volume_options,
};

pub(crate) fn show_plot_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let is_3d_mode = matches!(
        app.active_plot_type,
        PlotType::Sphere | PlotType::Surface | PlotType::Volume | PlotType::PointCloud
    );

    match app.active_plot_type {
        PlotType::Volume => show_volume_options(app, ui),
        PlotType::Sphere => show_sphere_options(app, ui),
        PlotType::Surface => show_surface_options(app, ui),
        PlotType::PointCloud => show_point_cloud_options(app, ui),
        PlotType::Line => show_line_options(app, ui),
        PlotType::Heatmap => show_heatmap_options(app, ui),
    }

    if is_3d_mode {
        show_3d_view_controls(app, ui);
    }
}

fn show_3d_view_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.sphere_auto_rotate, "Auto Rotate");
        if ui
            .icon_button(Icon::Reset, "Reset View")
            .on_hover_text("Reset 3D camera orientation")
            .clicked()
        {
            app.sphere_rotation_x = 0.25;
            app.sphere_rotation_y = 0.0;
            app.sphere_zoom = 2.5;
        }
        ui.selectable_label(app.show_hover_card, "Hover Card")
            .on_hover_text(if app.show_hover_card {
                "Hide hover card"
            } else {
                "Show hover card"
            })
            .clicked()
            .then(|| app.show_hover_card = !app.show_hover_card);
    });
}
