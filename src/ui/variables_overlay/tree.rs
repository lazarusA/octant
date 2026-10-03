use super::item::{MAX_ITEMS_PER_LEVEL, render_variable_list};
use super::row::{RowKind, allocate_row, paint_row};
use crate::data::{VariableInfo, VariableTreeGroup};
use egui::collapsing_header::CollapsingState;

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
    if !is_root {
        render_subgroup(ui, group, ctx);
        return;
    }

    if !group.variable_indices.is_empty() {
        if group.subgroups.is_empty() {
            // Only root variables: list them directly.
            render_variable_list(
                ui,
                &group.variable_indices,
                ctx.variables,
                ctx.selected_idx,
                &mut ctx.newly_selected_idx,
            );
        } else {
            // Root variables next to subgroups get their own collapsible "/" folder.
            let id = folder_id(ui, "", ctx.search_active);
            let count = group.variable_indices.len();
            render_folder(ui, id, ctx.search_active, "/", count, |ui| {
                render_variable_list(
                    ui,
                    &group.variable_indices,
                    ctx.variables,
                    ctx.selected_idx,
                    &mut ctx.newly_selected_idx,
                );
            });
        }
    }

    render_subgroups(ui, &group.subgroups, ctx);
}

pub fn render_subgroup(
    ui: &mut egui::Ui,
    subgroup: &VariableTreeGroup,
    ctx: &mut VariableTreeContext<'_>,
) {
    let id = folder_id(ui, &subgroup.full_path, ctx.search_active);
    let count = subgroup.total_variable_count();
    render_folder(ui, id, ctx.search_active, &subgroup.name, count, |ui| {
        // Direct variables first, then deeper subgroups (at most 100 each).
        render_variable_list(
            ui,
            &subgroup.variable_indices,
            ctx.variables,
            ctx.selected_idx,
            &mut ctx.newly_selected_idx,
        );
        render_subgroups(ui, &subgroup.subgroups, ctx);
    });
}

fn render_subgroups(
    ui: &mut egui::Ui,
    subgroups: &[VariableTreeGroup],
    ctx: &mut VariableTreeContext<'_>,
) {
    for subgroup in subgroups.iter().take(MAX_ITEMS_PER_LEVEL) {
        render_subgroup(ui, subgroup, ctx);
    }
    if subgroups.len() > MAX_ITEMS_PER_LEVEL {
        ui.label(
            egui::RichText::new(format!(
                "Showing 100 of {} folders. Use search to discover all.",
                subgroups.len()
            ))
            .small()
            .italics()
            .color(ui.visuals().weak_text_color()),
        );
    }
}

/// Folder open state is kept apart while searching, so search results always
/// start expanded and clearing the search restores the folders the user had open.
pub(super) fn folder_id(ui: &egui::Ui, full_path: &str, search_active: bool) -> egui::Id {
    ui.make_persistent_id(("var_tree_group", search_active, full_path))
}

/// Full-width folder row: clicking anywhere on it (chevron, icon or name) toggles it.
fn render_folder(
    ui: &mut egui::Ui,
    id: egui::Id,
    default_open: bool,
    name: &str,
    count: usize,
    add_body: impl FnOnce(&mut egui::Ui),
) {
    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let resp = allocate_row(ui);
    if resp.clicked() {
        state.toggle(ui);
        state.store(ui.ctx());
    }

    let mut buf = [0u8; 24];
    let detail = format_count(&mut buf, count);
    let kind = RowKind::Folder {
        open: state.is_open(),
    };
    paint_row(ui, &resp, kind, name, detail, false);

    state.show_body_indented(&resp, ui, add_body);
}

fn format_count(buf: &mut [u8; 24], count: usize) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{}", count);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
