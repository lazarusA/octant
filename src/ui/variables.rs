use crate::app::{AnimationRole, DimConfig, OctantApp, SpatialRole};

pub fn show_variables_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variables_overlay {
        return;
    }

    let screen_size = ctx.input(|i| i.viewport_rect().size());

    let width = (screen_size.x * 0.28).clamp(280.0, 480.0);
    let max_height = (screen_size.y * 0.65).clamp(250.0, 750.0);

    app.variables_overlay_width = width;

    let area_resp = egui::Area::new(egui::Id::new("octant_variables_area"))
        .fixed_pos(egui::pos2(
            canvas_rect.left() + 8.0,
            canvas_rect.top() + 8.0,
        ))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);

                egui::CollapsingHeader::new("📊 Variables")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.text_edit_singleline(&mut app.variable_search);
                        });
                        egui::ScrollArea::vertical()
                            .max_height(max_height)
                            .min_scrolled_height(max_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if app.is_loading {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);
                                        ui.spinner();

                                        ui.label(
                                            egui::RichText::new(
                                                "Inspecting store metadata in background...",
                                            )
                                            .italics(),
                                        );

                                        ui.add_space(10.0);
                                    });

                                    return;
                                }

                                let Some(metadata) = &app.active_dataset_metadata else {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);

                                        ui.label("No store metadata loaded yet.");

                                        ui.add_space(6.0);

                                        if ui.button("🔍 Fetch / Load Store Metadata").clicked() {
                                            app.inspect_active_store();
                                        }

                                        ui.add_space(10.0);
                                    });

                                    return;
                                };

                                if metadata.variables.is_empty() {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);

                                        ui.label("No variables found in this dataset store.");

                                        ui.add_space(6.0);

                                        if ui.button("🔍 Refresh Store Metadata").clicked() {
                                            app.inspect_active_store();
                                        }

                                        ui.add_space(10.0);
                                    });

                                    return;
                                }

                                let search = app.variable_search.to_lowercase();

                                for (idx, var_info) in metadata.variables.iter().enumerate() {
                                    if !search.is_empty()
                                        && !var_info.name.to_lowercase().contains(&search)
                                    {
                                        continue;
                                    }

                                    let is_selected = app.selected_variable_idx == idx;

                                    let label_text = if let Some(units) = &var_info.units {
                                        format!(
                                            "{}  [{}] ({})",
                                            var_info.name, var_info.data_type, units
                                        )
                                    } else {
                                        format!("{}  [{}]", var_info.name, var_info.data_type)
                                    };

                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            egui::RichText::new(label_text).strong(),
                                        )
                                        .clicked()
                                    {
                                        app.selected_variable_idx = idx;

                                        let rank = var_info.shape.len();

                                        app.dim_config = (0..rank)
                                            .map(|_| DimConfig {
                                                spatial: SpatialRole::None,
                                                animation: AnimationRole::None,
                                                active: false,
                                            })
                                            .collect();

                                        app.selected_dim_indices = vec![0; rank];

                                        app.selected_dim_ranges = var_info
                                            .shape
                                            .iter()
                                            .map(|&s| (0, (s as usize).saturating_sub(1)))
                                            .collect();
                                        app.spatial_dims.clear();
                                        app.animated_dim = None;

                                        app.show_variable_controls = true;

                                        if var_info.shape.len() <= 1 {
                                            app.line_plot_all_series = false;
                                            app.line_profile_dim_idx = 0;
                                            app.line_profile_slice_idx = 0;
                                        }
                                    }
                                }
                            });
                    });
            });
        });
    app.variables_overlay_width = area_resp.response.rect.width();
}
