use crate::{app::OctantApp, data::VariableTreeGroup};

pub fn show_variables_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variables_overlay {
        return;
    }

    let screen_size = ctx.input(|i| i.viewport_rect().size());

    let width = (screen_size.x * 0.28).clamp(280.0, 520.0);
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
                            if !app.variable_search.is_empty() && ui.small_button("✕").clicked() {
                                app.variable_search.clear();
                            }
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

                                let metadata = match &app.active_dataset_metadata {
                                    Some(meta) => meta,
                                    None => {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(10.0);

                                            ui.label("No store metadata loaded yet.");

                                            ui.add_space(6.0);

                                            if ui.button("🔍 Fetch / Load Store Metadata").clicked()
                                            {
                                                let target = app.store_target_input.clone();
                                                app.submit_or_activate_source(
                                                    &target,
                                                    Some(app.selected_store_kind),
                                                );
                                            }

                                            ui.add_space(10.0);
                                        });

                                        return;
                                    }
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

                                if app.cached_variable_tree.is_none() {
                                    app.cached_variable_tree =
                                        Some(metadata.build_variable_tree());
                                }
                                let tree = app.cached_variable_tree.as_ref().unwrap();
                                let search_query = app.variable_search.trim();
                                let search_active = !search_query.is_empty();

                                let filtered_tree;
                                let root_group_ref = if search_active {
                                    filtered_tree = tree.filter(search_query, &metadata.variables);
                                    filtered_tree.as_ref()
                                } else {
                                    Some(tree)
                                };

                                let Some(root_group) = root_group_ref else {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "No variables matching '{}'",
                                                search_query
                                            ))
                                            .italics(),
                                        );
                                        ui.add_space(10.0);
                                    });
                                    return;
                                };

                                let mut newly_selected_idx: Option<usize> = None;

                                render_tree_group(
                                    ui,
                                    root_group,
                                    &metadata.variables,
                                    app.selected_variable_idx,
                                    search_active,
                                    &mut newly_selected_idx,
                                    true,
                                );

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

fn render_tree_group(
    ui: &mut egui::Ui,
    group: &VariableTreeGroup,
    variables: &[crate::data::VariableInfo],
    selected_idx: usize,
    search_active: bool,
    newly_selected_idx: &mut Option<usize>,
    is_root: bool,
) {
    if is_root {
        // At root: render subgroups first, then root variables (or vice-versa)
        for subgroup in &group.subgroups {
            render_subgroup(
                ui,
                subgroup,
                variables,
                selected_idx,
                search_active,
                newly_selected_idx,
            );
        }

        if !group.variable_indices.is_empty() {
            if !group.subgroups.is_empty() {
                ui.add_space(4.0);
                ui.separator();
                ui.label(egui::RichText::new("Root Variables").small().weak());
            }

            for &idx in &group.variable_indices {
                if let Some(var_info) = variables.get(idx) {
                    render_variable_row(ui, var_info, idx, selected_idx, newly_selected_idx);
                }
            }
        }
    } else {
        render_subgroup(
            ui,
            group,
            variables,
            selected_idx,
            search_active,
            newly_selected_idx,
        );
    }
}

fn render_subgroup(
    ui: &mut egui::Ui,
    subgroup: &VariableTreeGroup,
    variables: &[crate::data::VariableInfo],
    selected_idx: usize,
    search_active: bool,
    newly_selected_idx: &mut Option<usize>,
) {
    let header_id = ui.make_persistent_id(("var_tree_group", &subgroup.full_path));
    let total_count = subgroup.total_variable_count();
    let header_title = format!("📁 {} ({})", subgroup.name, total_count);

    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        header_id,
        search_active || subgroup.full_path.split('/').count() <= 1,
    )
    .show_header(ui, |ui| {
        ui.label(egui::RichText::new(header_title).strong());
    })
    .body(|ui| {
        ui.indent(
            ui.make_persistent_id(("var_tree_body", &subgroup.full_path)),
            |ui| {
                for nested_sub in &subgroup.subgroups {
                    render_subgroup(
                        ui,
                        nested_sub,
                        variables,
                        selected_idx,
                        search_active,
                        newly_selected_idx,
                    );
                }

                for &idx in &subgroup.variable_indices {
                    if let Some(var_info) = variables.get(idx) {
                        render_variable_row(ui, var_info, idx, selected_idx, newly_selected_idx);
                    }
                }
            },
        );
    });
}

fn render_variable_row(
    ui: &mut egui::Ui,
    var_info: &crate::data::VariableInfo,
    idx: usize,
    selected_idx: usize,
    newly_selected_idx: &mut Option<usize>,
) {
    let is_selected = selected_idx == idx;
    let leaf_name = var_info.leaf_name();

    let label_text = match &var_info.units {
        Some(units) if !units.is_empty() => format!("📄 {}  ({})", leaf_name, units),
        _ => format!("📄 {}", leaf_name),
    };

    let response = ui.selectable_label(is_selected, egui::RichText::new(label_text).strong());

    let response = response.on_hover_ui(|ui| {
        ui.label(egui::RichText::new(&var_info.name).strong());
        if let Some(group) = var_info.group_path() {
            ui.label(format!("Group: 📁 {}", group.replace('/', " ❯ ")));
        }
        ui.label(format!("Type: [{}]", var_info.data_type));
        ui.label(format!("Shape: {:?}", var_info.shape));
        if let Some(desc) = &var_info.long_name {
            ui.label(format!("Description: {}", desc));
        }
    });

    if response.clicked() {
        *newly_selected_idx = Some(idx);
    }
}
