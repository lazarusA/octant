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

    // Safety fallback: if any other plot was active while 2D pyramid resampling is enabled (or volume exceeds limit), revert to 2D Plane
    if (app.enable_pyramid_resampling && app.active_plot_type != PlotType::Heatmap)
        || ((app.active_plot_type == PlotType::Volume
            || app.active_plot_type == PlotType::PointCloud)
            && !is_volume_allowed)
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

        let pyramid_disabled_reason = "Disabled: 2D Pyramid Resampling active".to_string();

        let options = [
            (
                PlotType::Heatmap,
                "🗺️ 2D Plane (Flatmap)",
                true,
                String::new(),
            ),
            (
                PlotType::Line,
                "📈 1D Line Chart",
                !app.enable_pyramid_resampling,
                if app.enable_pyramid_resampling {
                    pyramid_disabled_reason.clone()
                } else {
                    String::new()
                },
            ),
            (
                PlotType::Sphere,
                "🌍 3D Globe (Sphere)",
                !app.enable_pyramid_resampling,
                if app.enable_pyramid_resampling {
                    pyramid_disabled_reason.clone()
                } else {
                    String::new()
                },
            ),
            (
                PlotType::Surface,
                "⛰️ 3D Surface / Blocks",
                !app.enable_pyramid_resampling,
                if app.enable_pyramid_resampling {
                    pyramid_disabled_reason.clone()
                } else {
                    String::new()
                },
            ),
            (
                PlotType::Volume,
                "☁️ 3D Volume Raycasting",
                is_volume_allowed,
                if app.enable_pyramid_resampling {
                    pyramid_disabled_reason.clone()
                } else if !is_3d_available {
                    "Requires 3D Data".to_string()
                } else if !is_size_allowed {
                    format!("Disabled: {:.0} MB > 128 MB GPU limit", vol_mb)
                } else {
                    String::new()
                },
            ),
            (
                PlotType::PointCloud,
                "✨ 3D Point Cloud",
                is_volume_allowed,
                if app.enable_pyramid_resampling {
                    pyramid_disabled_reason
                } else if !is_3d_available {
                    "Requires 3D Data".to_string()
                } else if !is_size_allowed {
                    format!("Disabled: {:.0} MB > 128 MB GPU limit", vol_mb)
                } else {
                    String::new()
                },
            ),
        ];

        for (plot_type, label, enabled, disabled_reason) in options {
            let is_selected = app.active_plot_type == plot_type;
            if enabled {
                if ui.selectable_label(is_selected, label).clicked() {
                    app.active_plot_type = plot_type;
                    if let Some(wgpu_render_state) = &app.wgpu_render_state
                        && let Some(mdata) = &app.matrix_data
                    {
                        app.update_active_2d_renderer_data(&wgpu_render_state.queue, &mdata.values);
                    }
                    app.load_selected_variable_block();
                    ui.close();
                }
            } else {
                ui.add_enabled(
                    false,
                    egui::Label::new(format!("{} ({})", label, disabled_reason)),
                );
            }
        }
    });
}
