use crate::app::OctantApp;
use crate::plots::PlotType;

pub fn show_plot_type_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let (is_3d_available, is_size_allowed, vol_mb) = if let Some(meta) =
        &app.active_dataset_metadata
    {
        if let Some(v) = meta.variables.get(app.selected_variable_idx) {
            let has_3d = v.shape.len() >= 3 || v.dimension_names.len() >= 3;
            let vol_elements = crate::ui::variables_panel::calculate_selected_volume_elements(app);
            let size_ok = vol_elements <= crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
            let mb = (vol_elements * 4) as f64 / (1024.0 * 1024.0);
            (has_3d, size_ok, mb)
        } else {
            (false, false, 0.0)
        }
    } else {
        (false, false, 0.0)
    };
    let is_volume_allowed = is_3d_available && is_size_allowed && !app.enable_pyramid_resampling;

    let total_2d_elements = if let Some(mdata) = &app.matrix_data {
        mdata.width * mdata.height
    } else {
        crate::ui::variables_panel::calculate_selected_2d_elements(app)
    };
    let is_surface_allowed = total_2d_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS
        && !app.enable_pyramid_resampling;
    let surface_mb = (total_2d_elements * 4) as f64 / (1024.0 * 1024.0);

    // Safety fallback: if any plot is active with invalid data constraints, revert to 2D Plane
    if (app.enable_pyramid_resampling && app.active_plot_type != PlotType::Heatmap)
        || ((app.active_plot_type == PlotType::Volume
            || app.active_plot_type == PlotType::PointCloud)
            && !is_volume_allowed)
        || ((app.active_plot_type == PlotType::Sphere
            || app.active_plot_type == PlotType::Surface
            || app.active_plot_type == PlotType::Block)
            && !is_surface_allowed)
    {
        app.active_plot_type = PlotType::Heatmap;
    }

    let current_label = match app.active_plot_type {
        PlotType::Heatmap => "🌐 Plot: 2D Plane",
        PlotType::Line => "📈 Plot: 1D Line Chart",
        PlotType::Sphere => "🌐 Plot: 3D Globe",
        PlotType::Surface | PlotType::Block => "🌐 Plot: 3D Surface / Blocks",
        PlotType::Volume => "🌐 Plot: 3D Volume",
        PlotType::PointCloud => "🌐 Plot: 3D Point Cloud",
    };

    ui.menu_button(current_label, |ui| {
        ui.set_min_width(240.0);

        ui.label(
            egui::RichText::new("Select Visualization Projection")
                .small()
                .weak(),
        );
        ui.separator();

        let pyramid_disabled = app.enable_pyramid_resampling;
        let pyramid_reason = "Disabled: 2D Pyramid Resampling active";

        let options: [(PlotType, &str, bool, Option<std::borrow::Cow<'static, str>>); 6] = [
            (PlotType::Heatmap, "🗺️ 2D Plane (Flatmap)", true, None),
            (
                PlotType::Line,
                "📈 1D Line Chart",
                !pyramid_disabled,
                if pyramid_disabled {
                    Some(pyramid_reason.into())
                } else {
                    None
                },
            ),
            (
                PlotType::Sphere,
                "🌍 3D Globe (Sphere)",
                is_surface_allowed,
                if pyramid_disabled {
                    Some(pyramid_reason.into())
                } else if !is_surface_allowed {
                    Some(format!("Disabled: {:.0} MB > 3D mesh limit", surface_mb).into())
                } else {
                    None
                },
            ),
            (
                PlotType::Surface,
                "⛰️ 3D Surface / Blocks",
                is_surface_allowed,
                if pyramid_disabled {
                    Some(pyramid_reason.into())
                } else if !is_surface_allowed {
                    Some(format!("Disabled: {:.0} MB > 3D mesh limit", surface_mb).into())
                } else {
                    None
                },
            ),
            (
                PlotType::Volume,
                "☁️ 3D Volume Raycasting",
                is_volume_allowed,
                if pyramid_disabled {
                    Some(pyramid_reason.into())
                } else if !is_3d_available {
                    Some("Requires 3D Data".into())
                } else if !is_size_allowed {
                    Some(format!("Disabled: {:.0} MB > 128 MB GPU limit", vol_mb).into())
                } else {
                    None
                },
            ),
            (
                PlotType::PointCloud,
                "✨ 3D Point Cloud",
                is_volume_allowed,
                if pyramid_disabled {
                    Some(pyramid_reason.into())
                } else if !is_3d_available {
                    Some("Requires 3D Data".into())
                } else if !is_size_allowed {
                    Some(format!("Disabled: {:.0} MB > 128 MB GPU limit", vol_mb).into())
                } else {
                    None
                },
            ),
        ];

        for (plot_type, label, enabled, disabled_reason) in options {
            let is_selected = app.active_plot_type == plot_type;
            if enabled {
                if ui.selectable_label(is_selected, label).clicked() {
                    app.active_plot_type = plot_type;
                    app.load_selected_variable_block();
                    ui.close();
                }
            } else if let Some(reason) = disabled_reason {
                ui.add_enabled(false, egui::Label::new(format!("{} ({})", label, reason)));
            }
        }
    });
}
