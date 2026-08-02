use crate::app::OctantApp;

pub fn show_variables_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("📊 Variables", |ui| {
        ui.set_min_width(280.0);
        ui.label(egui::RichText::new("Extracted Metadata").strong());

        let mut var_changed = false;
        if let Some(metadata) = &app.active_dataset_metadata {
            ui.label(format!("Provider: {}", metadata.store_type));
            ui.label(format!("Store: {}", metadata.name));
            ui.label(format!("Variables: {}", metadata.variables.len()));

            if !metadata.variables.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("Select Active Variable:").strong());
                let old_var_idx = app.selected_variable_idx;

                let current_var_name = metadata
                    .variables
                    .get(app.selected_variable_idx)
                    .map(|v| v.name.as_str())
                    .unwrap_or("Select Variable");

                egui::ComboBox::from_id_salt("top_var_select")
                    .selected_text(current_var_name)
                    .show_ui(ui, |ui| {
                        for (idx, var_info) in metadata.variables.iter().enumerate() {
                            ui.selectable_value(&mut app.selected_variable_idx, idx, &var_info.name);
                        }
                    });

                if old_var_idx != app.selected_variable_idx {
                    var_changed = true;
                }

                if let Some(var_info) = metadata.variables.get(app.selected_variable_idx) {
                    ui.add_space(4.0);
                    ui.small(format!("DType: {}", var_info.data_type));
                    ui.small(format!("Shape: {:?}", var_info.shape));
                    ui.small(format!("Dimensions: {:?}", var_info.dimension_names));
                    ui.small(format!("Chunks: {:?}", var_info.chunk_shape));
                    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
                    ui.small(format!("File Size: {} bytes ({:.2} MB)", var_info.file_size, size_mb));
                }
            }
        } else {
            ui.label("No store metadata inspected.");
        }

        if var_changed {
            app.load_selected_variable_slice();
            ui.close_menu();
        }
    });
}
