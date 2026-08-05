use super::cache;
use crate::app::OctantApp;
use crate::plots::PlotType;

pub fn show_plot_controls_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    let is_3d_mode = app.active_plot_type == PlotType::Sphere
        || app.active_plot_type == PlotType::Surface
        || app.active_plot_type == PlotType::Volume
        || app.active_plot_type == PlotType::PointCloud;

    egui::Panel::top("octant_plot_controls_bar")
        .default_size(34.0)
        .show(ui, |ui| {
            ui.add_space(3.0);
            ui.horizontal_wrapped(|ui| {
                match app.active_plot_type {
                    PlotType::Volume => {
                        ui.label(egui::RichText::new("☁️ Volume:").strong().small());

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
                        ui.add(
                            egui::Slider::new(&mut app.volume_step_count, 16..=256).text("🌫 Steps"),
                        );

                        // Density slider (used in algorithms 0, 3, 4, 5, 6)
                        if app.volume_algorithm != 1 && app.volume_algorithm != 2 {
                            ui.add(
                                egui::Slider::new(&mut app.volume_opacity, 0.1..=10.0)
                                    .text("💧 Density"),
                            );
                        }

                        // Min Clip & Max Range sliders (used in algorithms 0, 1, 2, 6)
                        if app.volume_algorithm == 0
                            || app.volume_algorithm == 1
                            || app.volume_algorithm == 2
                            || app.volume_algorithm == 6
                        {
                            ui.separator();
                            if app.volume_algorithm != 2 {
                                ui.add(
                                    egui::Slider::new(&mut app.volume_cmin, 0.0..=100.0)
                                        .text("✂️ Min Clip"),
                                );
                            }
                            ui.add(
                                egui::Slider::new(&mut app.volume_cmax, 0.0..=100.0)
                                    .text("📊 Max Range"),
                            );
                        }

                        // Isovalue & Isorange sliders (used specifically in Solid Isosurface algorithm 1)
                        if app.volume_algorithm == 1 {
                            ui.separator();
                            ui.add(
                                egui::Slider::new(&mut app.volume_isovalue, -100.0..=100.0)
                                    .text("🎯 Isovalue"),
                            );
                            ui.add(
                                egui::Slider::new(&mut app.volume_isorange, 0.1..=20.0)
                                    .text("📏 Isorange"),
                            );
                        }
                    }
                    PlotType::Sphere => {
                        ui.label(egui::RichText::new("🌍 3D Globe:").strong().small());
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
                        ui.label(egui::RichText::new("⛰️ 3D Surface:").strong().small());
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
                        ui.label(egui::RichText::new("✨ Point Cloud:").strong().small());
                        ui.add(
                            egui::Slider::new(&mut app.point_cloud_size, 0.002..=0.10)
                                .text("✨ Size"),
                        );
                    }
                    PlotType::Heatmap | PlotType::Block => {
                        ui.label(
                            egui::RichText::new("🗺️ 2D Plane Heatmap Active")
                                .small()
                                .weak(),
                        );
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
            });
        });
}

pub fn show_bottom_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    if !app.show_bottom_bar {
        return;
    }

    egui::Panel::bottom("octant_bottom_bar")
        .exact_size(38.0)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // 1. Play / Pause Button for Timestep Animation across all plot types
                let play_text = if app.is_playing { "⏸" } else { "▶" };
                let play_button = egui::Button::new(egui::RichText::new(format!("{} {}", play_text, if app.is_playing { "Pause" } else { "Play" })).strong());
                if ui.add(play_button).clicked() {
                    app.is_playing = !app.is_playing;
                    app.last_step_time = std::time::Instant::now();
                }

                let max_steps = app
                    .matrix_data
                    .as_ref()
                    .map(|h| h.max_timesteps)
                    .unwrap_or(1);

                // 2. Prev Step
                if ui.button("◀").on_hover_text("Previous Step").clicked() {
                    if app.current_timestep > 0 {
                        app.current_timestep -= 1;
                    } else if max_steps > 0 {
                        app.current_timestep = max_steps - 1;
                    }
                    app.load_selected_variable_slice();
                }

                // 3. Next Step
                if ui.button("▶").on_hover_text("Next Step").clicked() {
                    if max_steps > 0 {
                        app.current_timestep = (app.current_timestep + 1) % max_steps;
                    }
                    app.load_selected_variable_slice();
                }

                // 4. Loop Toggle
                ui.checkbox(&mut app.loop_playback, "🔄 Loop");

                ui.separator();

                let status_text = if app.is_playing { "▶ Playing" } else { "⏸ Paused" };
                let status_color = if app.is_playing {
                    egui::Color32::from_rgb(255, 99, 71)
                } else {
                    egui::Color32::LIGHT_GRAY
                };
                ui.label(egui::RichText::new(status_text).small().color(status_color));

                ui.separator();

                // 6. Timestep timeline slider & Dimension-Agnostic Axis Reading
                let active_var_info = app
                    .active_dataset_metadata
                    .as_ref()
                    .and_then(|m| m.variables.get(app.selected_variable_idx));

                let active_anim_dim = active_var_info
                    .and_then(|v| v.dimension_names.first().cloned())
                    .unwrap_or_else(|| "time".to_string());

                let direct_coord_label = app
                    .active_dataset_metadata
                    .as_ref()
                    .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()))
                    .and_then(|coords| coords.get(app.current_timestep).cloned());

                let formatted_axis = if let Some(coord_str) = direct_coord_label {
                    coord_str
                } else {
                    let active_dim_name =
                        active_var_info.and_then(|v| v.dimension_names.first().cloned());
                    let active_units = active_var_info.and_then(|v| v.units.as_deref());
                    let time_start = active_var_info.and_then(|v| v.time_coverage_start.as_deref());
                    let temp_res = active_var_info.and_then(|v| v.temporal_resolution.as_deref());

                    crate::utils::units::format_axis_value(
                        app.current_timestep,
                        max_steps,
                        active_dim_name.as_deref(),
                        active_units,
                        time_start,
                        temp_res,
                        Some(&app.store_target_input),
                    )
                };

                let slider_max = max_steps.saturating_sub(1);
                ui.label(
                    egui::RichText::new(format!("📅 {}", formatted_axis))
                        .small()
                        .monospace()
                        .strong(),
                )
                .on_hover_text(format!(
                    "Step {} / {}",
                    app.current_timestep + 1,
                    max_steps
                ));

                let slider_res = ui.add(
                    egui::Slider::new(&mut app.current_timestep, 0..=slider_max)
                        .show_value(false)
                        .trailing_fill(true),
                );
                if slider_res.drag_stopped() || (slider_res.changed() && !app.is_playing) {
                    app.load_selected_variable_slice();
                }

                ui.separator();

                // 7. Playback speed slider
                ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
                    ui.label(egui::RichText::new("Playback Speed").strong());
                    ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
                });

                // 8. Bottom Right: Cache Menu Dropdown
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    cache::show_cache_menu(app, ui);
                });
            });
        });
}
