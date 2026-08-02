use crate::app::OctantApp;

pub fn show_variables_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    let menu_title = if let Some(metadata) = &app.active_dataset_metadata {
        if let Some(var) = metadata.variables.get(app.selected_variable_idx) {
            format!("📊 Variables ({})", var.name)
        } else {
            "📊 Variables".to_string()
        }
    } else {
        "📊 Variables".to_string()
    };

    ui.menu_button(menu_title, |ui| {
        ui.set_min_width(340.0);
        let max_height = (ui.ctx().screen_rect().height() * 0.8).max(200.0);

        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Dataset Variable Catalog").strong());
                ui.separator();

                let mut var_changed = false;
                if let Some(metadata) = &app.active_dataset_metadata {
                    if !metadata.variables.is_empty() {
                        ui.label(egui::RichText::new("Select Variable (Click to View Metadata):").small().strong());
                        ui.add_space(4.0);

                        for (idx, var_info) in metadata.variables.iter().enumerate() {
                            let is_selected = app.selected_variable_idx == idx;

                            ui.group(|ui| {
                                let label_text = format!("{}  [{}]", var_info.name, var_info.data_type);

                                if ui.selectable_label(is_selected, egui::RichText::new(label_text).strong()).clicked() {
                                    if app.selected_variable_idx != idx {
                                        app.selected_variable_idx = idx;
                                        var_changed = true;
                                    }
                                }

                                // Show detailed metadata on click / selection
                                if is_selected {
                                    ui.add_space(4.0);
                                    ui.small(format!("Shape: {:?}", var_info.shape));
                                    ui.small(format!("Chunk Shape: {:?}", var_info.chunk_shape));
                                    ui.small(format!("Dimensions: {:?}", var_info.dimension_names));
                                    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
                                    ui.small(format!("File Size: {:.2} MB ({} bytes)", size_mb, var_info.file_size));
                                }
                            });
                            ui.add_space(2.0);
                        }
                    } else {
                        ui.label("No variables found in store.");
                    }
                } else {
                    ui.label("No store metadata inspected.");
                }

                if var_changed {
                    app.load_selected_variable_slice();
                }
            });
    });
}
