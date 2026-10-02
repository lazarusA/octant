pub mod item;
pub mod search;
pub mod tree;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use search::render_search_bar;
use tree::{VariableTreeContext, render_tree_group};

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

                let header_id = ui.make_persistent_id("variables_overlay_header");
                let mut should_close = false;

                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    header_id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.icon(Icon::Variables, IconSize::Sm);
                    ui.label(egui::RichText::new("Variables").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .icon_button(Icon::Cross, "Close")
                            .on_hover_text("Close Variables Window")
                            .clicked()
                        {
                            should_close = true;
                        }
                    });
                })
                .body(|ui| {
                    render_search_bar(ui, &mut app.variable_search);

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

                                        if ui
                                            .icon_button(
                                                Icon::Search,
                                                "Fetch / Load Store Metadata",
                                            )
                                            .clicked()
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

                                    if ui
                                        .icon_button(Icon::Search, "Refresh Store Metadata")
                                        .clicked()
                                    {
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
                                }
                            }
                        });
                });

                if should_close {
                    app.show_variables_overlay = false;
                    if !app.show_variable_controls {
                        app.revert_selected_state_to_plotted();
                    }
                }
            });
        });
    app.variables_overlay_width = area_resp.response.rect.width();
}
