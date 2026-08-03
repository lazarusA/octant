use crate::app::OctantApp;
use super::{colormap, plot_type, status, store, variables};

pub fn show_top_bar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("octant_top_bar")
        .exact_height(34.0)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            egui::menu::bar(ui, |ui| {
                // Octant Brand Header
                ui.label(egui::RichText::new("📐 Octant").strong().heading());
                ui.separator();

                // Dropdown menus: Store, Catalog, Variables, Colormap, Plot Type
                store::show_store_menu(app, ui);

                if ui.button(egui::RichText::new("📚 Catalog").strong()).clicked() {
                    app.show_catalog_window = true;
                }

                variables::show_variables_menu(app, ui);
                colormap::show_colormap_menu(app, ui);
                plot_type::show_plot_type_menu(app, ui);

                if app.active_dataset_metadata.is_some() {
                    // Right status info and controls toggle on far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let panel_label = if app.show_right_panel { "🎛️ Controls ◀" } else { "🎛️ Controls ▶" };
                        if ui.button(egui::RichText::new(panel_label).strong()).on_hover_text("Toggle Right Variable Controls Panel").clicked() {
                            app.show_right_panel = !app.show_right_panel;
                        }
                        ui.separator();
                        status::show_status_bar(app, ui);
                    });
                } else {
                    status::show_status_bar(app, ui);
                }
            });
        });

    show_secondary_toolbar(app, ctx);
}

fn show_secondary_toolbar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("octant_secondary_toolbar")
        .exact_height(32.0)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎨 Clipping & Bounds:").strong().small());
                ui.separator();

                // 1. NaN Color Picker & Transparent Toggle
                ui.checkbox(&mut app.use_nan_color, "Custom NaN Color")
                    .on_hover_text("If unchecked, NaN and Inf values render transparently.");
                if app.use_nan_color {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.nan_color);
                }

                ui.separator();

                // 2. Low Clip Color Picker & Toggle
                ui.checkbox(&mut app.use_lowclip, "Low Clip")
                    .on_hover_text("If unchecked, values < cmin render using the colormap minimum value.");
                if app.use_lowclip {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.lowclip_color);
                }

                ui.separator();

                // 3. High Clip Color Picker & Toggle
                ui.checkbox(&mut app.use_highclip, "High Clip")
                    .on_hover_text("If unchecked, values > cmax render using the colormap maximum value.");
                if app.use_highclip {
                    ui.color_edit_button_rgba_unmultiplied(&mut app.highclip_color);
                }

                ui.separator();

                // 4. Color Range Bounds (cmin, cmax) & Fixed Bounds Lock
                ui.label("Min:");
                ui.add(egui::DragValue::new(&mut app.color_range_min).speed(0.1));

                ui.label("Max:");
                ui.add(egui::DragValue::new(&mut app.color_range_max).speed(0.1));

                let lock_label = if app.lock_color_bounds { "🔒 Bounds Locked" } else { "🔓 Bounds Dynamic" };
                if ui.selectable_label(app.lock_color_bounds, lock_label)
                    .on_hover_text("Lock min/max bounds so color mapping remains fixed across all timesteps and slices.")
                    .clicked()
                {
                    app.lock_color_bounds = !app.lock_color_bounds;
                }

                if ui.button("↺ Reset").on_hover_text("Reset bounds to current slice data min/max").clicked() {
                    if let Some(mdata) = &app.matrix_data {
                        app.color_range_min = mdata.min_val;
                        app.color_range_max = mdata.max_val;
                        app.volume_cmin = mdata.min_val;
                        app.volume_cmax = mdata.max_val;
                    }
                }

                ui.separator();

                // 5. Color Scale Selection
                let is_valid_log = app.color_range_min >= -1e-15 && app.color_range_max > 0.0;
                if !is_valid_log && app.active_scale_type == 1 {
                    app.active_scale_type = 0;
                }

                ui.label(egui::RichText::new("📈 Scale:").strong().small());
                egui::ComboBox::from_id_salt("color_scale_dropdown")
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
                    ui.label("Param:");
                    ui.add(egui::DragValue::new(&mut app.scale_param).speed(0.01).range(0.0001..=100.0));
                }
            });
        });
}
