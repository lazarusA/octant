use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::settings::composite::show_composite_controls;
use crate::ui::settings::plot_2d::{show_heatmap_options, show_line_options};
use crate::ui::settings::plot_3d::{
    show_point_cloud_options, show_sphere_options, show_surface_options, show_volume_options,
};

/// Options of `plot_type` alone: its mode, geometry and composite channels.
pub(crate) fn show_plot_options(app: &mut OctantApp, ui: &mut egui::Ui, plot_type: PlotType) {
    match plot_type {
        PlotType::Volume => show_volume_options(app, ui),
        PlotType::Sphere => show_sphere_options(app, ui),
        PlotType::Surface => show_surface_options(app, ui),
        PlotType::PointCloud => show_point_cloud_options(app, ui),
        // Lines draw one band: no composite.
        PlotType::Line => return show_line_options(app, ui),
        PlotType::Heatmap => show_heatmap_options(app, ui),
    }

    if app.has_rgb_bands() || !app.layers.base.composite.channel_configs.is_empty() {
        show_composite_controls(app, ui);
    } else if plot_type == PlotType::Heatmap && app.layers.base.composite.enabled {
        app.layers.base.composite.enabled = false;
    }
}
