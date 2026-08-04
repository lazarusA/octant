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
        let screen_size = ui.ctx().input(|i| i.viewport_rect()).size();
        let target_width = (screen_size.x * 0.28).clamp(280.0, 480.0);
        let max_height = (screen_size.y * 0.65).clamp(250.0, 750.0);

        ui.set_min_width(target_width);

        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Dataset Variables").strong());
                ui.small("Select a variable to open the Right Control Panel with sliders and plot button.");
                ui.separator();

                if app.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.spinner();
                        ui.label(egui::RichText::new("Inspecting store metadata in background...").italics());
                        ui.add_space(10.0);
                    });
                } else if let Some(metadata) = &app.active_dataset_metadata {
                    if !metadata.variables.is_empty() {
                        for (idx, var_info) in metadata.variables.iter().enumerate() {
                            let is_selected = app.selected_variable_idx == idx;

                            let label_text = if let Some(units) = &var_info.units {
                                format!("{}  [{}] ({})", var_info.name, var_info.data_type, units)
                            } else {
                                format!("{}  [{}]", var_info.name, var_info.data_type)
                            };

                            if ui.selectable_label(is_selected, egui::RichText::new(label_text).strong()).clicked() {
                                app.selected_variable_idx = idx;
                                app.selected_dim_indices = vec![0; var_info.shape.len()];
                                app.selected_dim_ranges = var_info.shape.iter().map(|&s| (0, (s as usize).saturating_sub(1))).collect();
                                app.show_right_panel = true;
                                ui.close();

                            }
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
            });
    });
}
