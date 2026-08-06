use crate::app::OctantApp;
use crate::plots::PlotType;

/// Anchored to the left edge of the canvas area, just below the top bar.
/// Stores its own width so Variable Controls can position to the right without overlap.
pub fn show_settings_window(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_settings_panel {
        app.settings_overlay_width = 0.0;
        return;
    }

    let area_resp = egui::Area::new(egui::Id::new("octant_settings_area"))
        .fixed_pos(egui::pos2(
            canvas_rect.left() + 8.0,
            canvas_rect.top() + 8.0,
        ))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.set_max_width(280.0);
                    egui::CollapsingHeader::new("⚙️ Settings")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_plot_options(app, ui);
                            ui.separator();
                            show_clipping_bounds(app, ui);
                        });
                });
        });

    // Store width for next frame so Variable Controls can position to the right.
    app.settings_overlay_width = area_resp.response.rect.width();
}

fn show_plot_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let is_3d_mode = app.active_plot_type == PlotType::Sphere
        || app.active_plot_type == PlotType::Surface
        || app.active_plot_type == PlotType::Volume
        || app.active_plot_type == PlotType::PointCloud;

    match app.active_plot_type {
        PlotType::Volume => {
            ui.label(egui::RichText::new("☁️ Volume").strong().small());

            let algo_label = match app.volume_algorithm {
                0 => "☁️ Volume Raymarching",
                1 => "🎯 Solid Isosurface (WIP)",
                2 => "⚡ Maximum Intensity (MIP)",
                3 => "🌫 Absorption RGBA (WIP)",
                4 => "✨ Additive RGBA (WIP)",
                5 => "🎨 Indexed RGBA (WIP)",
                _ => "📐 Shaded Contours (WIP)",
            };
            ui.menu_button(egui::RichText::new(algo_label).small(), |ui| {
                let algos = [
                    (0, "☁️ Volume Raymarching (Default)"),
                    (1, "🎯 Solid Isosurface (WIP / Experimental)"),
                    (2, "⚡ Maximum Intensity (MIP)"),
                    (3, "🌫 Absorption RGBA (WIP / Experimental)"),
                    (4, "✨ Additive RGBA (WIP / Experimental)"),
                    (5, "🎨 Indexed RGBA (WIP / Experimental)"),
                    (6, "📐 Shaded Contours (WIP / Experimental)"),
                ];
                for (id, label) in algos {
                    if ui
                        .selectable_label(app.volume_algorithm == id, label)
                        .clicked()
                    {
                        app.volume_algorithm = id;
                        ui.close();
                    }
                }
            });

            ui.separator();
            ui.add(egui::Slider::new(&mut app.volume_step_count, 16..=256).text("🌫 Steps"));

            if app.volume_algorithm != 1 && app.volume_algorithm != 2 {
                ui.add(egui::Slider::new(&mut app.volume_opacity, 0.1..=10.0).text("💧 Density"));
            }

            if app.volume_algorithm == 0
                || app.volume_algorithm == 1
                || app.volume_algorithm == 2
                || app.volume_algorithm == 6
            {
                ui.separator();
                if app.volume_algorithm != 2 {
                    ui.add(
                        egui::Slider::new(&mut app.volume_cmin, 0.0..=100.0).text("✂️ Min Clip"),
                    );
                }
                ui.add(egui::Slider::new(&mut app.volume_cmax, 0.0..=100.0).text("📊 Max Range"));
            }

            if app.volume_algorithm == 1 {
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut app.volume_isovalue, -100.0..=100.0).text("🎯 Isovalue"),
                );
                ui.add(egui::Slider::new(&mut app.volume_isorange, 0.1..=20.0).text("📏 Isorange"));
            }
        }
        PlotType::Sphere => {
            ui.label(egui::RichText::new("🌍 3D Globe").strong().small());
            let style_label = match app.sphere_mode {
                0 => "🌍 Smooth Globe",
                1 => "🌋 Smooth Terrain",
                2 => "📐 Flat Steps",
                _ => "🧱 3D Radial Legos",
            };
            if ui
                .button(egui::RichText::new(style_label).small())
                .clicked()
            {
                app.sphere_mode = (app.sphere_mode + 1) % 4;
            }
            if app.sphere_mode > 0 {
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut app.sphere_displacement_strength, 0.0..=5.0)
                        .text("🌋 Height"),
                );
            }
        }
        PlotType::Surface => {
            ui.label(egui::RichText::new("⛰️ 3D Surface").strong().small());
            let style_label = match app.surface_mode {
                0 => "🌊 Smooth Terrain",
                1 => "📐 Flat Steps",
                _ => "🧱 3D Lego Cubes",
            };
            if ui
                .button(egui::RichText::new(style_label).small())
                .clicked()
            {
                app.surface_mode = (app.surface_mode + 1) % 3;
            }
            ui.separator();
            ui.add(
                egui::Slider::new(&mut app.surface_displacement_strength, 0.0..=5.0)
                    .text("⛰️ Height"),
            );
        }
        PlotType::PointCloud => {
            ui.label(egui::RichText::new("✨ Point Cloud").strong().small());
            ui.add(egui::Slider::new(&mut app.point_cloud_size, 0.002..=0.10).text("✨ Size"));
        }
        PlotType::Line => {
            ui.label(egui::RichText::new("📈 1D Line Plot").strong().small());
            let mut use_flat = app.active_colormap == 999;
            if ui
                .checkbox(&mut use_flat, "🎨 Solid Flat Line Color")
                .changed()
            {
                if use_flat {
                    app.active_colormap = 999;
                } else {
                    app.active_colormap = 0;
                }
            }

            let profile_controls = app.matrix_data.as_ref().map_or((false, 0usize), |matrix| {
                (
                    matrix.width > 1 || matrix.height > 1,
                    matrix.width.max(matrix.height),
                )
            });

            if profile_controls.0 {
                ui.separator();
                ui.label(egui::RichText::new("🧭 Line Profile").small().weak());

                let mut selected_dim_idx = app.line_profile_dim_idx;
                egui::ComboBox::from_id_salt("line_profile_dim_selector")
                    .selected_text(if selected_dim_idx == 0 {
                        "Along X / Longitude"
                    } else {
                        "Along Y / Latitude"
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(selected_dim_idx == 0, "Along X / Longitude")
                            .clicked()
                        {
                            selected_dim_idx = 0;
                        }
                        if ui
                            .selectable_label(selected_dim_idx == 1, "Along Y / Latitude")
                            .clicked()
                        {
                            selected_dim_idx = 1;
                        }
                    });
                if selected_dim_idx != app.line_profile_dim_idx {
                    app.line_profile_dim_idx = selected_dim_idx;
                }

                let mut all_series = app.line_plot_all_series;
                if ui
                    .checkbox(&mut all_series, "📈 All Lines Series")
                    .changed()
                {
                    app.line_plot_all_series = all_series;
                }

                if !app.line_plot_all_series {
                    let max_idx = if app.line_profile_dim_idx == 0 {
                        app.matrix_data
                            .as_ref()
                            .map_or(0, |matrix| matrix.height.saturating_sub(1))
                    } else {
                        app.matrix_data
                            .as_ref()
                            .map_or(0, |matrix| matrix.width.saturating_sub(1))
                    };
                    if max_idx > 0 {
                        let mut slice_idx = app.line_profile_slice_idx;
                        if ui
                            .add(
                                egui::Slider::new(&mut slice_idx, 0..=max_idx)
                                    .text("🧪 Profile Index"),
                            )
                            .changed()
                        {
                            app.line_profile_slice_idx = slice_idx;
                        }
                    } else {
                        ui.label("Single profile available.");
                    }
                }
            }
        }
        PlotType::Heatmap | PlotType::Block => {
            ui.label(egui::RichText::new("🗺️ 2D Heatmap").small().weak());
        }
    }

    if is_3d_mode {
        ui.separator();
        ui.checkbox(&mut app.sphere_auto_rotate, "🎥 Rotate");
        if ui
            .button("🔄 Reset View")
            .on_hover_text("Reset 3D camera orientation")
            .clicked()
        {
            app.sphere_rotation_x = 0.25;
            app.sphere_rotation_y = 0.0;
            app.sphere_zoom = 2.5;
        }
    }
}

fn show_clipping_bounds(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.checkbox(&mut app.use_nan_color, "Custom NaN Color")
        .on_hover_text("If unchecked, NaN/Inf values render transparently.");
    if app.use_nan_color {
        ui.color_edit_button_rgba_unmultiplied(&mut app.nan_color);
    }

    ui.add_space(2.0);
    ui.checkbox(&mut app.use_lowclip, "Low Clip")
        .on_hover_text("Values < cmin clipped to this color.");
    if app.use_lowclip {
        ui.color_edit_button_rgba_unmultiplied(&mut app.lowclip_color);
    }

    ui.add_space(2.0);
    ui.checkbox(&mut app.use_highclip, "High Clip")
        .on_hover_text("Values > cmax clipped to this color.");
    if app.use_highclip {
        ui.color_edit_button_rgba_unmultiplied(&mut app.highclip_color);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Min:");
        ui.add(egui::DragValue::new(&mut app.color_range_min).speed(0.1));
        ui.label("Max:");
        ui.add(egui::DragValue::new(&mut app.color_range_max).speed(0.1));
    });

    let lock_label = if app.lock_color_bounds {
        "🔒 Locked"
    } else {
        "🔓 Dynamic"
    };
    if ui
        .selectable_label(app.lock_color_bounds, lock_label)
        .on_hover_text("Lock min/max so color mapping stays fixed across timesteps.")
        .clicked()
    {
        app.lock_color_bounds = !app.lock_color_bounds;
    }

    if ui
        .button("↺ Reset")
        .on_hover_text("Reset bounds to current slice min/max")
        .clicked()
        && let Some(mdata) = &app.matrix_data
    {
        app.color_range_min = mdata.min_val;
        app.color_range_max = mdata.max_val;
        app.volume_cmin = mdata.min_val;
        app.volume_cmax = mdata.max_val;
    }

    ui.add_space(4.0);
    ui.label(egui::RichText::new("📈 Scale").strong());
    let is_valid_log = app.color_range_min >= -1e-15 && app.color_range_max > 0.0;
    if !is_valid_log && app.active_scale_type == 1 {
        app.active_scale_type = 0;
    }
    egui::ComboBox::from_id_salt("settings_color_scale_dropdown")
        .selected_text(match app.active_scale_type {
            1 => "Logarithmic",
            2 => "Symlog",
            3 => "Sqrt",
            4 => "Exponential",
            _ => "Linear",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut app.active_scale_type, 0, "Linear");
            ui.add_enabled_ui(is_valid_log, |ui| {
                ui.selectable_value(&mut app.active_scale_type, 1, "Logarithmic")
                    .on_hover_text(if is_valid_log {
                        "Log scale (non-negative data)"
                    } else {
                        "Disabled: requires min ≥ 0. Use Symlog for negative data."
                    });
            });
            ui.selectable_value(&mut app.active_scale_type, 2, "Symlog");
            ui.selectable_value(&mut app.active_scale_type, 3, "Sqrt");
            ui.selectable_value(&mut app.active_scale_type, 4, "Exponential");
        });

    if app.active_scale_type == 1 || app.active_scale_type == 2 || app.active_scale_type == 4 {
        ui.horizontal(|ui| {
            ui.label("Param:");
            ui.add(
                egui::DragValue::new(&mut app.scale_param)
                    .speed(0.01)
                    .range(0.0001..=100.0),
            );
        });
    }

    ui.add_space(2.0);
    ui.toggle_value(&mut app.is_categorical, "🎨 Categorical")
        .on_hover_text("Discrete colorbar (auto-detects unique values or 10 equal bins).");
}
