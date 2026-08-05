use crate::app::{OctantApp, StoreKind};

pub fn show_left_panel(app: &mut OctantApp, ui: &mut egui::Ui) {
    if !app.show_left_panel {
        return;
    }

    let panel_width = 280.0;
    egui::Panel::left("octant_left_store_panel")
        .resizable(false)
        .default_size(panel_width)
        .show(ui, |ui| {
            ui.heading("🗂️ Stores");
            ui.add_space(4.0);
            ui.label("Choose a data source and load metadata into the current session.");
            ui.separator();

            let mut selected = app.selected_store_kind;
            egui::ComboBox::from_id_salt("left_store_kind_select")
                .selected_text(match selected {
                    StoreKind::RemoteZarr => "🌐 Remote Zarr",
                    StoreKind::LocalZarr => "📁 Local Zarr",
                    StoreKind::RemoteIcechunk => "🧊 Remote Icechunk",
                    StoreKind::LocalIcechunk => "🧊 Local Icechunk",
                    StoreKind::ProceduralRandom => "🎲 Procedural Random",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, StoreKind::RemoteZarr, "🌐 Remote Zarr (HTTP/S3)");
                    ui.selectable_value(&mut selected, StoreKind::LocalZarr, "📁 Local Zarr (FileSystem)");
                    ui.selectable_value(&mut selected, StoreKind::RemoteIcechunk, "🧊 Remote Icechunk (HTTP/S3)");
                    ui.selectable_value(&mut selected, StoreKind::LocalIcechunk, "🧊 Local Icechunk (FileSystem)");
                    ui.selectable_value(&mut selected, StoreKind::ProceduralRandom, "🎲 Procedural Random Test");
                });

            if selected != app.selected_store_kind {
                app.selected_store_kind = selected;
                match app.selected_store_kind {
                    StoreKind::RemoteZarr => {
                        app.store_target_input = "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string();
                    }
                    StoreKind::LocalZarr => {
                        app.store_target_input = "./data/sample_dataset.zarr".to_string();
                    }
                    StoreKind::RemoteIcechunk => {
                        app.store_target_input = "https://s3.amazonaws.com/icechunk-demo/repository".to_string();
                    }
                    StoreKind::LocalIcechunk => {
                        app.store_target_input = "./data/icechunk_repo".to_string();
                    }
                    StoreKind::ProceduralRandom => {
                        app.store_target_input = "procedural://random".to_string();
                    }
                }
            }

            ui.add_space(6.0);
            ui.label(egui::RichText::new("Target URL / Path").strong());
            let res = ui.text_edit_singleline(&mut app.store_target_input);
            if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                app.inspect_active_store();
            }

            ui.add_space(6.0);
            let btn_label = if app.is_loading { "⏳ Fetching..." } else { "🔍 Load Metadata" };
            if ui.add_enabled(!app.is_loading, egui::Button::new(egui::RichText::new(btn_label).strong())).clicked() {
                app.inspect_active_store();
            }

            ui.add_space(6.0);
            if ui.button(egui::RichText::new("📚 Open Catalog").strong()).clicked() {
                app.show_catalog_window = true;
            }

            ui.separator();
            ui.heading("🎨 Clipping & Bounds");
            ui.add_space(4.0);
            ui.label("Fine-tune color mapping and clipping for the active dataset.");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height().max(180.0) - 120.0)
                .show(ui, |ui| {
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
                        && let Some(mdata) = &app.matrix_data {
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
                    egui::ComboBox::from_id_salt("left_color_scale_dropdown")
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

            ui.separator();
            ui.collapsing("About this store", |ui| {
                if let Some(metadata) = &app.active_dataset_metadata {
                    ui.label(format!("Provider: {}", metadata.store_type));
                    ui.label(format!("Dataset: {}", metadata.name));
                    ui.label(format!("Variables: {}", metadata.variables.len()));
                } else {
                    ui.label("No dataset metadata loaded yet.");
                }
            });
        });
}

pub fn show_store_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    if ui.button(egui::RichText::new("🌐 Store").strong()).clicked() {
        app.show_left_panel = true;
    }
}
