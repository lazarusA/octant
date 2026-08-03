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

                // 2. Dimension Index Selection & Sliders (x, y, z, time, level/depth)
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎛️ Dimension Selection Sliders").strong());
                    ui.small("Adjust sliders for dimension indices (time, level, lat, lon). Unmapped extra dimensions will be collapsed to slice/volume.");
                    ui.separator();

                    let dim_count = var_info.shape.len();
                    if app.selected_dim_indices.len() != dim_count {
                        app.selected_dim_indices = vec![0; dim_count];
                    }

                    for (i, shape_dim) in var_info.shape.iter().enumerate() {
                        let dim_size = *shape_dim as usize;
                        let max_idx = dim_size.saturating_sub(1);
                        let dim_name = var_info
                            .dimension_names
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("dim_{}", i));

                        let mut current_idx = app.selected_dim_indices.get(i).copied().unwrap_or(0).min(max_idx);

                        // Look up coordinate label if available
                        let coord_label = dim_coords
                            .get(&dim_name.to_lowercase())
                            .and_then(|coords| coords.get(current_idx).cloned())
                            .unwrap_or_else(|| {
                                crate::utils::units::format_axis_value(
                                    current_idx,
                                    dim_size,
                                    Some(&dim_name),
                                    var_info.units.as_deref(),
                                    var_info.time_coverage_start.as_deref(),
                                    var_info.temporal_resolution.as_deref(),
                                    app.active_dataset_metadata.as_ref().map(|m| m.name.as_str()),
                                )
                            });

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}:", dim_name)).strong().small());
                                ui.label(egui::RichText::new(format!("{} (index {} / {})", coord_label, current_idx, dim_size)).small().monospace());
                            });

                            let slider_res = ui.add(
                                egui::Slider::new(&mut current_idx, 0..=max_idx)
                                    .show_value(false)
                                    .trailing_fill(true),
                            );

                            if slider_res.changed() {
                                if let Some(elem) = app.selected_dim_indices.get_mut(i) {
                                    *elem = current_idx;
                                }
                                // If this is dimension 0 (time), sync current_timestep
                                if i == 0 {
                                    app.current_timestep = current_idx;
                                }
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
