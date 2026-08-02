use crate::app::OctantApp;

pub fn show_variables_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let menu_title = if app.is_loading {
        "📊 Variables (⏳ Loading...)".to_string()
    } else if let Some(metadata) = &app.active_dataset_metadata {
        if let Some(var) = metadata.variables.get(app.selected_variable_idx) {
            format!("📊 Variables ({})", var.name)
        } else {
            "📊 Variables (Empty)".to_string()
        }
    } else {
        "📊 Variables (Not Loaded)".to_string()
    };

    ui.menu_button(menu_title, |ui| {
        let screen_size = ui.ctx().screen_rect().size();
        let target_width = (screen_size.x * 0.38).clamp(380.0, 620.0);
        let max_height = (screen_size.y * 0.70).clamp(300.0, 850.0);

        ui.set_min_width(target_width);

        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Dataset Variable Catalog").strong());
                ui.separator();

                let mut var_changed = false;

                if app.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.spinner();
                        ui.label(egui::RichText::new("Inspecting store metadata in background...").italics());
                        ui.add_space(10.0);
                    });
                } else if let Some(metadata) = &app.active_dataset_metadata {
                    if !metadata.variables.is_empty() {
                        ui.label(egui::RichText::new("Select Variable (Click to View Metadata):").small().strong());
                        ui.add_space(4.0);

                        for (idx, var_info) in metadata.variables.iter().enumerate() {
                            let is_selected = app.selected_variable_idx == idx;

                            ui.group(|ui| {
                                let label_text = if let Some(units) = &var_info.units {
                                    format!("{}  [{}] ({})", var_info.name, var_info.data_type, units)
                                } else {
                                    format!("{}  [{}]", var_info.name, var_info.data_type)
                                };

                                if ui.selectable_label(is_selected, egui::RichText::new(label_text).strong()).clicked() {
                                    if app.selected_variable_idx != idx {
                                        app.selected_variable_idx = idx;
                                        var_changed = true;
                                    }
                                }

                                // Show detailed metadata on click / selection
                                if is_selected {
                                    ui.add_space(4.0);

                                    if let Some(long_name) = &var_info.long_name {
                                        ui.small(format!("Description: {}", long_name));
                                    }
                                    if let Some(units) = &var_info.units {
                                        ui.small(format!("Units: {}", units));
                                    }

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
                                        ui.collapsing("All Attributes (.zattrs)", |ui| {
                                            for (k, v) in &var_info.attributes {
                                                ui.small(format!("{}: {}", k, v));
                                            }
                                        });
                                    }
                                }
                            });
                            ui.add_space(2.0);
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label("No variables found in this dataset store.");
                            ui.add_space(6.0);
                            if ui.button("🔍 Refresh Store Metadata").clicked() {
                                app.inspect_active_store();
                            }
                            ui.add_space(10.0);
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label("No store metadata loaded yet.");
                        ui.add_space(6.0);
                        if ui.button("🔍 Fetch / Load Store Metadata").clicked() {
                            app.inspect_active_store();
                        }
                        ui.add_space(10.0);
                    });
                }

                if var_changed {
                    app.load_selected_variable_slice();
                }
            });
    });
}
