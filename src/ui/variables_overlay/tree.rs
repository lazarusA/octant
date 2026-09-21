use super::item::{MAX_ITEMS_PER_LEVEL, render_variable_list};
use crate::data::{VariableInfo, VariableTreeGroup};
use crate::ui::icons::{Icon, UiIconExt};

pub struct VariableTreeContext<'a> {
    pub variables: &'a [VariableInfo],
    pub selected_idx: usize,
    pub search_active: bool,
    pub newly_selected_idx: Option<usize>,
}

pub fn render_tree_group(
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

                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    root_header_id,
                    ctx.search_active,
                )
                .show_header(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Folder, 12.0);
                        let mut buf = [0u8; 32];
                        let label_text = format_root_count(&mut buf, root_count);
                        ui.label(egui::RichText::new(label_text).strong());
                    });
                })
                .body(|ui| {
                    ui.indent(ui.make_persistent_id("var_tree_root_vars_body"), |ui| {
                        render_variable_list(
                            ui,
                            &group.variable_indices,
                            ctx.variables,
                            ctx.selected_idx,
                            &mut ctx.newly_selected_idx,
                        );
                    });
                });
            } else {
                // When only root variables exist, display them directly up to MAX_ITEMS_PER_LEVEL
                render_variable_list(
                    ui,
                    &group.variable_indices,
                    ctx.variables,
                    ctx.selected_idx,
                    &mut ctx.newly_selected_idx,
                );
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
                    "Showing 100 of {} folders. Use search to discover all.",
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

pub fn render_subgroup(
    ui: &mut egui::Ui,
    subgroup: &VariableTreeGroup,
    ctx: &mut VariableTreeContext<'_>,
) {
    let header_id = ui.make_persistent_id(("var_tree_group", &subgroup.full_path));
    let total_count = subgroup.total_variable_count();

    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        header_id,
        ctx.search_active, // Folders are closed by default, expanded only during search
    )
    .show_header(ui, |ui| {
        ui.horizontal(|ui| {
            ui.icon(Icon::Folder, 12.0);
            let mut buf = [0u8; 96];
            let label_text = format_subgroup_title(&mut buf, &subgroup.name, total_count);
            ui.label(egui::RichText::new(label_text).strong());
        });
    })
    .body(|ui| {
        ui.indent(
            ui.make_persistent_id(("var_tree_body", &subgroup.full_path)),
            |ui| {
                // Direct variables in this subgroup first (at most 100)
                render_variable_list(
                    ui,
                    &subgroup.variable_indices,
                    ctx.variables,
                    ctx.selected_idx,
                    &mut ctx.newly_selected_idx,
                );

                // Followed by deeper nested subgroups (at most 100)
                let sub_count = subgroup.subgroups.len();
                for nested_sub in subgroup.subgroups.iter().take(MAX_ITEMS_PER_LEVEL) {
                    render_subgroup(ui, nested_sub, ctx);
                }
                if sub_count > MAX_ITEMS_PER_LEVEL {
                    ui.label(
                        egui::RichText::new(format!(
                            "Showing 100 of {} folders. Use search to discover all.",
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

fn format_root_count(buf: &mut [u8; 32], count: usize) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "/ ({})", count);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("/")
}

fn format_subgroup_title<'a>(buf: &'a mut [u8; 96], name: &str, count: usize) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{} ({})", name, count);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
