use crate::app::OctantApp;
use crate::plots::PlotType;

pub fn show_settings_panel(app: &mut OctantApp, ui: &mut egui::Ui) {
    if !app.show_settings_panel {
        return;
    }

    let panel_width = 320.0;
    egui::Panel::right("octant_settings_panel")
        .resizable(false)
        .default_size(panel_width)
        .show(ui, |ui| {
            ui.set_min_width(panel_width);
            ui.set_max_width(panel_width);
            ui.heading("⚙️ Settings");
            ui.add_space(4.0);
            ui.label("Playback, plot options, and clipping controls live here.");
            ui.separator();

            ui.add_space(6.0);
            ui.scope(|ui| {
                ui.label(egui::RichText::new("Plot Options").strong());
                ui.add_space(4.0);

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

                        if app.volume_algorithm == 0 || app.volume_algorithm == 1 || app.volume_algorithm == 2 || app.volume_algorithm == 6 {
                            ui.separator();
                            if app.volume_algorithm != 2 {
                                ui.add(egui::Slider::new(&mut app.volume_cmin, 0.0..=100.0).text("✂️ Min Clip"));
                            }
                            ui.add(egui::Slider::new(&mut app.volume_cmax, 0.0..=100.0).text("📊 Max Range"));
                        }

                        if app.volume_algorithm == 1 {
                            ui.separator();
                            ui.add(egui::Slider::new(&mut app.volume_isovalue, -100.0..=100.0).text("🎯 Isovalue"));
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
                        if ui.button(egui::RichText::new(style_label).small()).clicked() {
                            app.sphere_mode = (app.sphere_mode + 1) % 4;
                        }
                        if app.sphere_mode > 0 {
                            ui.separator();
                            ui.add(egui::Slider::new(&mut app.sphere_displacement_strength, 0.0..=5.0).text("🌋 Height"));
                        }
                    }
                    PlotType::Surface => {
                        ui.label(egui::RichText::new("⛰️ 3D Surface").strong().small());
                        let style_label = match app.surface_mode {
                            0 => "🌊 Smooth Terrain",
                            1 => "📐 Flat Steps",
                            _ => "🧱 3D Lego Cubes",
                        };
                        if ui.button(egui::RichText::new(style_label).small()).clicked() {
                            app.surface_mode = (app.surface_mode + 1) % 3;
                        }
                        ui.separator();
                        ui.add(egui::Slider::new(&mut app.surface_displacement_strength, 0.0..=5.0).text("⛰️ Height"));
                    }
                    PlotType::PointCloud => {
                        ui.label(egui::RichText::new("✨ Point Cloud").strong().small());
                        ui.add(egui::Slider::new(&mut app.point_cloud_size, 0.002..=0.10).text("✨ Size"));
                    }
                    PlotType::Heatmap | PlotType::Block => {
                        ui.label(egui::RichText::new("🗺️ 2D Plane Heatmap Active").small().weak());
                    }
                }

                if is_3d_mode {
                    ui.separator();
                    ui.checkbox(&mut app.sphere_auto_rotate, "🎥 Rotate");
                    if ui.button("🔄 Reset View").on_hover_text("Reset 3D camera orientation").clicked() {
                        app.sphere_rotation_x = 0.25;
                        app.sphere_rotation_y = 0.0;
                        app.sphere_zoom = 2.5;
                    }
                }
            });

            ui.add_space(6.0);
            ui.scope(|ui| {
                ui.label(egui::RichText::new("🎨 Clipping & Bounds").strong());
                ui.add_space(4.0);
                ui.label("Fine-tune color mapping and clipping for the active dataset.");
                ui.separator();

                ui.checkbox(&mut app.use_nan_color, "Custom NaN Color")
                    .on_hover_text("If unchecked, NaN and Inf values render transparently.");
                if app.use_nan_color {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.nan_color);
                }

                ui.add_space(4.0);
                ui.checkbox(&mut app.use_lowclip, "Low Clip")
                    .on_hover_text("If unchecked, values < cmin render using the colormap minimum value.");
                if app.use_lowclip {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.lowclip_color);
                }

                ui.add_space(4.0);
                ui.checkbox(&mut app.use_highclip, "High Clip")
                    .on_hover_text("If unchecked, values > cmax render using the colormap maximum value.");
                if app.use_highclip {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.highclip_color);
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Min:");
                    ui.add(egui::DragValue::new(&mut app.color_range_min).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Max:");
                    ui.add(egui::DragValue::new(&mut app.color_range_max).speed(0.1));
                });

                let lock_label = if app.lock_color_bounds { "🔒 Bounds Locked" } else { "🔓 Bounds Dynamic" };
                if ui.selectable_label(app.lock_color_bounds, lock_label)
                    .on_hover_text("Lock min/max bounds so color mapping remains fixed across all timesteps and slices.")
                    .clicked()
                {
                    app.lock_color_bounds = !app.lock_color_bounds;
                }

                if ui.button("↺ Reset").on_hover_text("Reset bounds to current slice data min/max").clicked()
                    && let Some(mdata) = &app.matrix_data
                {
                    app.color_range_min = mdata.min_val;
                    app.color_range_max = mdata.max_val;
                    app.volume_cmin = mdata.min_val;
                    app.volume_cmax = mdata.max_val;
                }

                ui.add_space(6.0);
                let is_valid_log = app.color_range_min >= -1e-15 && app.color_range_max > 0.0;
                if !is_valid_log && app.active_scale_type == 1 {
                    app.active_scale_type = 0;
                }

                ui.label(egui::RichText::new("📈 Scale").strong());
                egui::ComboBox::from_id_salt("settings_color_scale_dropdown")
                    .selected_text(match app.active_scale_type {
                        1 => "Logarithmic",
                        2 => "Symlog (Log-Offset)",
                        3 => "Sqrt (Diverging)",
                        4 => "Exponential",
                        _ => "Linear",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.active_scale_type, 0, "Linear");

                        ui.add_enabled_ui(is_valid_log, |ui| {
                            ui.selectable_value(&mut app.active_scale_type, 1, "Logarithmic")
                                .on_hover_text(if is_valid_log {
                                    "Logarithmic scale (for non-negative data)"
                                } else {
                                    "Disabled: Logarithmic scale requires non-negative data (min >= 0). Use Symlog for data with negative values."
                                });
                        });

                        ui.selectable_value(&mut app.active_scale_type, 2, "Symlog (Log-Offset)");
                        ui.selectable_value(&mut app.active_scale_type, 3, "Sqrt (Diverging)");
                        ui.selectable_value(&mut app.active_scale_type, 4, "Exponential");
                    });

                if app.active_scale_type == 1 || app.active_scale_type == 2 || app.active_scale_type == 4 {
                    ui.horizontal(|ui| {
                        ui.label("Param:");
                        ui.add(egui::DragValue::new(&mut app.scale_param).speed(0.01).range(0.0001..=100.0));
                    });
                }

                ui.add_space(4.0);
                ui.toggle_value(&mut app.is_categorical, "🎨 Categorical")
                    .on_hover_text("Enable Categorical / Discrete colorbar (auto-detects unique values, or defaults to 10 equal bins)");
            });
        });
}
