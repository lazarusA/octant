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
                            let search_has_text = !app.variable_search.is_empty();
                            let edit_width = if search_has_text {
                                (ui.available_width() - 26.0).max(60.0)
                            } else {
                                ui.available_width()
                            };

                            ui.add(
                                egui::TextEdit::singleline(&mut app.variable_search)
                                    .hint_text("Search variables...")
                                    .desired_width(edit_width),
                            );

                            if search_has_text
                                && ui
                                    .small_button("x")
                                    .on_hover_text("Clear search")
                                    .clicked()
                            {
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

                                let tree = app
                                    .cached_variable_tree
                                    .get_or_insert_with(|| metadata.build_variable_tree());
                                let search_query = app.variable_search.trim();
                                let search_active = !search_query.is_empty();

                                let filtered_tree;
                                let root_group_ref = if search_active {
                                    filtered_tree = tree.filter(search_query, &metadata.variables);
                                    filtered_tree.as_ref()
                                } else {
                                    Some(&*tree)
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

                                let mut tree_ctx = VariableTreeContext {
                                    variables: &metadata.variables,
                                    selected_idx: app.selected_variable_idx,
                                    search_active,
                                    newly_selected_idx: None,
                                };

                                render_tree_group(ui, root_group, &mut tree_ctx, true);

                                if let Some(idx) = tree_ctx.newly_selected_idx {
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

const MAX_ITEMS_PER_LEVEL: usize = 100;

struct VariableTreeContext<'a> {
    variables: &'a [crate::data::VariableInfo],
    selected_idx: usize,
    search_active: bool,
    newly_selected_idx: Option<usize>,
}

fn render_tree_group(
    ui: &mut egui::Ui,
    group: &VariableTreeGroup,
    ctx: &mut VariableTreeContext<'_>,
    is_root: bool,
) {
    if is_root {
        let has_subgroups = !group.subgroups.is_empty();
        let has_root_vars = !group.variable_indices.is_empty();

        if has_root_vars {
            if has_subgroups {
                // If there are both subgroups and root variables, make root variables collapsible (starts collapsed)
                let root_header_id = ui.make_persistent_id("var_tree_root_vars");
                let root_count = group.variable_indices.len();
                let header_title = format!("📁 / ({})", root_count);

                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    root_header_id,
                    ctx.search_active,
                )
                .show_header(ui, |ui| {
                    ui.label(egui::RichText::new(header_title).strong());
                })
                .body(|ui| {
                    ui.indent(ui.make_persistent_id("var_tree_root_vars_body"), |ui| {
                        render_variable_list(ui, &group.variable_indices, ctx);
                    });
                });
            } else {
                // When only root variables exist, display them directly up to MAX_ITEMS_PER_LEVEL
                render_variable_list(ui, &group.variable_indices, ctx);
            }
        }

        // Render root-level subgroups (at most 100)
        let sub_count = group.subgroups.len();
        for subgroup in group.subgroups.iter().take(MAX_ITEMS_PER_LEVEL) {
            render_subgroup(ui, subgroup, ctx);
        }
        if sub_count > MAX_ITEMS_PER_LEVEL {
            ui.label(
                egui::RichText::new(format!(
                    "Showing 100 of {} folders. Use 🔍 search to discover all.",
                    sub_count
                ))
                .small()
                .italics()
                .color(ui.visuals().weak_text_color()),
            );
        }
    } else {
        render_subgroup(ui, group, ctx);
    }
}

fn render_subgroup(
    ui: &mut egui::Ui,
    subgroup: &VariableTreeGroup,
    ctx: &mut VariableTreeContext<'_>,
) {
    let header_id = ui.make_persistent_id(("var_tree_group", &subgroup.full_path));
    let total_count = subgroup.total_variable_count();
    let header_title = format!("📁 {} ({})", subgroup.name, total_count);

    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        header_id,
        ctx.search_active, // Folders are closed by default, expanded only during search
    )
    .show_header(ui, |ui| {
        ui.label(egui::RichText::new(header_title).strong());
    })
    .body(|ui| {
        ui.indent(
            ui.make_persistent_id(("var_tree_body", &subgroup.full_path)),
            |ui| {
                // Direct variables in this subgroup first (at most 100)
                render_variable_list(ui, &subgroup.variable_indices, ctx);

                // Followed by deeper nested subgroups (at most 100)
                let sub_count = subgroup.subgroups.len();
                for nested_sub in subgroup.subgroups.iter().take(MAX_ITEMS_PER_LEVEL) {
                    render_subgroup(ui, nested_sub, ctx);
                }
                if sub_count > MAX_ITEMS_PER_LEVEL {
                    ui.label(
                        egui::RichText::new(format!(
                            "Showing 100 of {} folders. Use 🔍 search to discover all.",
                            sub_count
                        ))
                        .small()
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                    );
                }
            },
        );
    });
}

fn render_variable_list(ui: &mut egui::Ui, indices: &[usize], ctx: &mut VariableTreeContext<'_>) {
    let total = indices.len();
    for &idx in indices.iter().take(MAX_ITEMS_PER_LEVEL) {
        if let Some(var_info) = ctx.variables.get(idx) {
            render_variable_row(
                ui,
                var_info,
                idx,
                ctx.selected_idx,
                &mut ctx.newly_selected_idx,
            );
        }
    }
    if total > MAX_ITEMS_PER_LEVEL {
        ui.label(
            egui::RichText::new(format!(
                "Showing 100 of {} variables in this folder. Use 🔍 search to discover all.",
                total
            ))
            .small()
            .italics()
            .color(ui.visuals().weak_text_color()),
        );
    }
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
