use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::icons::{Icon, UiIconExt};

/// Map a PlotType to its corresponding vector Icon.
#[inline]
pub fn plot_type_icon(plot_type: PlotType) -> Icon {
    match plot_type {
        PlotType::Heatmap => Icon::PlotPlane,
        PlotType::Line => Icon::PlotLine,
        PlotType::Sphere => Icon::PlotGlobe,
        PlotType::Surface => Icon::PlotSurface,
        PlotType::Volume => Icon::PlotVolume,
        PlotType::PointCloud => Icon::PlotPointCloud,
    }
}

pub fn show_plot_type_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let (is_3d_available, is_size_allowed, vol_mb) = if let Some(v) = app
        .plotted_variable_info()
        .or_else(|| app.selected_variable_info())
    {
        let has_3d = v.shape.len() >= 3 || v.dimension_names.len() >= 3;
        let vol_elements = if let Some(vdata) = &app.volume_data {
            vdata
                .width
                .saturating_mul(vdata.height)
                .saturating_mul(vdata.depth)
        } else {
            crate::ui::variables_panel::calculate_selected_volume_elements(app)
        };
        let size_ok = vol_elements <= crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
        let mb = (vol_elements as f64 * 4.0) / (1024.0 * 1024.0);
        (has_3d, size_ok, mb)
    } else {
        (false, false, 0.0)
    };
    let is_discrete_grid = app
        .dim_config
        .iter()
        .any(|c| c.spatial == crate::app::SpatialRole::Grid);

    let supported_plots = app
        .matrix_data
        .as_ref()
        .map(|m| m.grid.supported_plot_types());
    let grid_name = app
        .matrix_data
        .as_ref()
        .map(|m| m.grid.name())
        .unwrap_or("active");

    let is_volume_allowed = (is_3d_available || app.volume_data.is_some())
        && is_size_allowed
        && !app.enable_pyramid_resampling
        && supported_plots
            .is_none_or(|plots| plots.contains(&PlotType::Volume) && !is_discrete_grid);

    let total_2d_elements = if let Some(mdata) = &app.matrix_data {
        mdata.width.saturating_mul(mdata.height)
    } else {
        crate::ui::variables_panel::calculate_selected_2d_elements(app)
    };
    let is_surface_allowed = total_2d_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS
        && !app.enable_pyramid_resampling;
    let surface_mb = (total_2d_elements as f64 * 4.0) / (1024.0 * 1024.0);

    // Safety fallback: revert to Heatmap only if the currently active plot lacks valid GPU data or pyramid is on
    if (app.enable_pyramid_resampling && app.active_plot_type != PlotType::Heatmap)
        || ((app.active_plot_type == PlotType::Volume
            || app.active_plot_type == PlotType::PointCloud)
            && app.volume_data.is_none())
        || ((app.active_plot_type == PlotType::Sphere || app.active_plot_type == PlotType::Surface)
            && app.sphere_renderer.is_none()
            && app.matrix_data.is_none())
    {
        app.active_plot_type = PlotType::Heatmap;
    }

    let current_icon = plot_type_icon(app.active_plot_type);
    let current_label = app.active_plot_type.display_name();

    ui.icon_menu_button(current_icon, current_label, |ui| {
        ui.set_min_width(220.0);

        ui.label(
            egui::RichText::new("Select Visualization Projection")
                .small()
                .weak(),
        );
        ui.separator();

        let pyramid_disabled = app.enable_pyramid_resampling;

        let plot_items = [
            (
                PlotType::Heatmap,
                Icon::PlotPlane,
                PlotType::Heatmap.display_name(),
                true,
            ),
            (
                PlotType::Line,
                Icon::PlotLine,
                PlotType::Line.display_name(),
                !pyramid_disabled,
            ),
            (
                PlotType::Sphere,
                Icon::PlotGlobe,
                PlotType::Sphere.display_name(),
                is_surface_allowed,
            ),
            (
                PlotType::Surface,
                Icon::PlotSurface,
                PlotType::Surface.display_name(),
                is_surface_allowed,
            ),
            (
                PlotType::Volume,
                Icon::PlotVolume,
                PlotType::Volume.display_name(),
                is_volume_allowed,
            ),
            (
                PlotType::PointCloud,
                Icon::PlotPointCloud,
                PlotType::PointCloud.display_name(),
                is_volume_allowed,
            ),
        ];

        for (plot_type, icon, label, enabled) in plot_items {
            let is_supported = supported_plots.is_none_or(|plots| plots.contains(&plot_type));
            let is_enabled = enabled && is_supported;
            let is_selected = app.active_plot_type == plot_type;

            if is_enabled {
                let clicked = ui
                    .horizontal(|ui| {
                        ui.icon(icon, 14.0);
                        ui.selectable_label(is_selected, label).clicked()
                    })
                    .inner;
                if clicked {
                    app.active_plot_type = plot_type;
                    app.load_selected_variable_block();
                    ui.close();
                }
            } else {
                let reason = if !is_supported {
                    format!("Unsupported for {} grid", grid_name)
                } else if pyramid_disabled {
                    "Disabled: 2D Pyramid Resampling active".to_string()
                } else if (plot_type == PlotType::Volume || plot_type == PlotType::PointCloud)
                    && !is_3d_available
                {
                    "Requires 3D Data".to_string()
                } else if plot_type == PlotType::Sphere || plot_type == PlotType::Surface {
                    format!("Disabled: {:.0} MB > 128 MB GPU limit", surface_mb)
                } else {
                    format!("Disabled: {:.0} MB > 128 MB GPU limit", vol_mb)
                };

                ui.horizontal(|ui| {
                    ui.icon_colored(icon, 14.0, ui.visuals().weak_text_color());
                    ui.add_enabled(false, egui::Label::new(format!("{} ({})", label, reason)));
                });
            }
        }
    });
}
