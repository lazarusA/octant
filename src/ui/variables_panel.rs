use crate::app::OctantApp;

pub fn show_right_panel(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_right_panel || app.active_dataset_metadata.is_none() {
        return;
    }

    egui::SidePanel::right("variable_control_panel")
        .resizable(true)
        .default_width(340.0)
        .min_width(280.0)
        .max_width(550.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Header bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📊 Variable Controls").strong().heading());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").on_hover_text("Close Panel").clicked() {
                        app.show_right_panel = false;
                    }
                });
            });
            ui.separator();

            let (var_info, dim_coords) = if let Some(meta) = &app.active_dataset_metadata {
                if let Some(v) = meta.variables.get(app.selected_variable_idx) {
                    (v.clone(), meta.dimension_coordinates.clone())
                } else {
                    ui.label("No variable selected.");
                    return;
                }
            } else {
                return;
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                // 1. Selected Variable Overview & Metadata (Moved here accompanying variable)
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&var_info.name).strong().size(16.0));
                        ui.label(egui::RichText::new(format!("[{}]", var_info.data_type)).small().weak());
                    });

                    if let Some(units) = &var_info.units {
                        ui.small(format!("Units: {}", units));
                    }
                    if let Some(long_name) = &var_info.long_name {
                        ui.small(format!("Description: {}", long_name));
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Metadata Specs").strong().small());

                    ui.small(format!("Shape: {:?}", var_info.shape));
                    ui.small(format!("Chunk Shape: {:?}", var_info.chunk_shape));
                    ui.small(format!("Dimensions: {:?}", var_info.dimension_names));

                    if let (Some(start), Some(end)) = (&var_info.time_coverage_start, &var_info.time_coverage_end) {
                        let start_clean = start.split('T').next().unwrap_or(start);
                        let end_clean = end.split('T').next().unwrap_or(end);
                        ui.small(format!("Time Coverage: {} to {}", start_clean, end_clean));
                    }
                    if let Some(res) = &var_info.temporal_resolution {
                        ui.small(format!("Temporal Resolution: {}", res));
                    }

                    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
                    ui.small(format!("File Size: {:.2} MB ({} bytes)", size_mb, var_info.file_size));

                    if !var_info.attributes.is_empty() {
                        ui.add_space(2.0);
                        ui.collapsing("All Attributes (.zattrs)", |ui| {
                            for (k, v) in &var_info.attributes {
                                ui.small(format!("{}: {}", k, v));
                            }
                        });
                    }
                });

                ui.add_space(8.0);

                // 2. Dimension Index Selection & Range Sliders (Dual Thumbs: Start and End per dimension)
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎛️ Dimension Selection & Slice Range Sliders").strong());
                    ui.small("Select start and end index thumbs for each dimension (time, level, lat, lon). The selected ranges will be used for plotting.");
                    ui.separator();

                    let dim_count = var_info.shape.len();
                    if app.selected_dim_indices.len() != dim_count {
                        app.selected_dim_indices = vec![0; dim_count];
                    }
                    if app.selected_dim_ranges.len() != dim_count {
                        app.selected_dim_ranges = var_info
                            .shape
                            .iter()
                            .map(|&s| (0, (s as usize).saturating_sub(1)))
                            .collect();
                    }

                    for (i, shape_dim) in var_info.shape.iter().enumerate() {
                        let dim_size = *shape_dim as usize;
                        let max_idx = dim_size.saturating_sub(1);
                        let dim_name = var_info
                            .dimension_names
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("dim_{}", i));

                        let (mut start_idx, mut end_idx) = app
                            .selected_dim_ranges
                            .get(i)
                            .copied()
                            .unwrap_or((0, max_idx));

                        start_idx = start_idx.min(max_idx);
                        end_idx = end_idx.clamp(start_idx, max_idx);

                        // Look up start coordinate label
                        let start_coord = dim_coords
                            .get(&dim_name.to_lowercase())
                            .and_then(|coords| coords.get(start_idx).cloned())
                            .unwrap_or_else(|| {
                                crate::utils::units::format_axis_value(
                                    start_idx,
                                    dim_size,
                                    Some(&dim_name),
                                    var_info.units.as_deref(),
                                    var_info.time_coverage_start.as_deref(),
                                    var_info.temporal_resolution.as_deref(),
                                    app.active_dataset_metadata.as_ref().map(|m| m.name.as_str()),
                                )
                            });

                        // Look up end coordinate label
                        let end_coord = dim_coords
                            .get(&dim_name.to_lowercase())
                            .and_then(|coords| coords.get(end_idx).cloned())
                            .unwrap_or_else(|| {
                                crate::utils::units::format_axis_value(
                                    end_idx,
                                    dim_size,
                                    Some(&dim_name),
                                    var_info.units.as_deref(),
                                    var_info.time_coverage_start.as_deref(),
                                    var_info.temporal_resolution.as_deref(),
                                    app.active_dataset_metadata.as_ref().map(|m| m.name.as_str()),
                                )
                            });

                        let selected_count = end_idx.saturating_sub(start_idx) + 1;

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}:", dim_name)).strong().small());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(format!("({} / {} steps)", selected_count, dim_size)).small().weak());
                                });
                            });

                            ui.small(format!("Range: {} ➔ {}", start_coord, end_coord));
                            ui.add_space(2.0);

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Start Thumb:").small());
                                let start_slider = ui.add(
                                    egui::Slider::new(&mut start_idx, 0..=max_idx)
                                        .show_value(true)
                                        .trailing_fill(true),
                                );
                                if start_slider.changed() && start_idx > end_idx {
                                    end_idx = start_idx;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("End Thumb:  ").small());
                                let end_slider = ui.add(
                                    egui::Slider::new(&mut end_idx, 0..=max_idx)
                                        .show_value(true)
                                        .trailing_fill(true),
                                );
                                if end_slider.changed() && end_idx < start_idx {
                                    start_idx = end_idx;
                                }
                            });

                            // Synchronize state
                            if let Some(r) = app.selected_dim_ranges.get_mut(i) {
                                *r = (start_idx, end_idx);
                            }
                            if let Some(idx_ref) = app.selected_dim_indices.get_mut(i) {
                                *idx_ref = start_idx;
                            }
                            if i == 0 {
                                app.current_timestep = start_idx;
                            }
                        });

                        ui.add_space(4.0);
                    }
                });


                ui.add_space(10.0);

                // 3. Dedicated Plot Button
                let plot_btn = egui::Button::new(egui::RichText::new("📊 Plot Data").strong().size(15.0));
                if ui.add_sized([ui.available_width(), 38.0], plot_btn).clicked() {
                    app.load_selected_variable_slice();
                }
            });
        });
}
