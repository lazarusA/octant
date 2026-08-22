use crate::app::OctantApp;

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

                                let variables = match &app.active_dataset_metadata {
                                    Some(meta) => &meta.variables,
                                    None => {
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
                                    }
                                };

                                if variables.is_empty() {
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

                                let mut newly_selected_idx: Option<usize> = None;

                                for (idx, var_info) in variables.iter().enumerate() {
                                    if !search.is_empty()
                                        && !var_info.name.to_lowercase().contains(&search)
                                    {
                                        continue;
                                    }

                                    let is_selected = app.selected_variable_idx == idx;

                                    let label_text = if let Some(units) = &var_info.units {
                                        format!(
                                            "{}  ({})",
                                            var_info.name, units
                                        )
                                    } else {
                                        var_info.name.to_string()
                                    };

                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            egui::RichText::new(label_text).strong(),
                                        )
                                        .clicked()
                                    {
                                        newly_selected_idx = Some(idx);
                                    }
                                }

                                if let Some(idx) = newly_selected_idx {
                                    app.selected_variable_idx = idx;
                                    app.reset_colorbar_label();

                                    if let Some(meta) = &app.active_dataset_metadata
                                        && let Some(var_info) = meta.variables.get(idx).cloned()
                                    {
                                        crate::ui::variables_panel::init_variable_dimension_defaults(
                                            app, &var_info,
                                        );

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
