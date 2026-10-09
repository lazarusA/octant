use super::nav::{NodeKey, row_id};
use super::row::{RowKind, allocate_row, paint_row};
use super::tree::VariableTreeContext;
use crate::data::VariableInfo;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};

pub const MAX_ITEMS_PER_LEVEL: usize = 100;
/// Room the "Add as overlay" button takes at a row's right end.
const OVERLAY_BUTTON: f32 = 24.0;

/// The "Add as overlay" button at the right end of `row`.
fn overlay_button(ui: &mut egui::Ui, row: egui::Rect) -> egui::Response {
    let side = row.height().min(OVERLAY_BUTTON);
    let rect = egui::Rect::from_center_size(
        egui::pos2(row.right() - OVERLAY_BUTTON * 0.5, row.center().y),
        egui::vec2(side, side),
    );
    let button = ToolbarButton::new(Icon::Layers, "Add as overlay")
        .compact(true)
        .icon_size(IconSize::Sm);
    ui.put(rect, button)
}

pub fn render_variable_list(
    ui: &mut egui::Ui,
    indices: &[usize],
    ctx: &mut VariableTreeContext<'_>,
) {
    let total = indices.len();
    for &idx in indices.iter().take(MAX_ITEMS_PER_LEVEL) {
        if let Some(var_info) = ctx.variables.get(idx) {
            render_variable_row(ui, var_info, idx, ctx);
        }
    }
    truncation_note(ui, total, "variables in this folder");
}

/// Muted note under a level listing more than [`MAX_ITEMS_PER_LEVEL`] items.
pub fn truncation_note(ui: &mut egui::Ui, total: usize, what: &str) {
    if total <= MAX_ITEMS_PER_LEVEL {
        return;
    }
    let text =
        format!("Showing {MAX_ITEMS_PER_LEVEL} of {total} {what}. Use search to discover all.");
    ui.label(
        egui::RichText::new(text)
            .small()
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

fn render_variable_row(
    ui: &mut egui::Ui,
    var_info: &VariableInfo,
    idx: usize,
    ctx: &mut VariableTreeContext<'_>,
) {
    let is_selected = ctx.selected_idx == idx;
    let units = var_info.units.as_deref().unwrap_or("");

    let resp = allocate_row(ui, row_id(NodeKey::Variable(idx), ctx.search_active));
    // Over the button the row itself is not hovered, so test the pointer.
    let offer_overlay = ctx.can_overlay && ui.rect_contains_pointer(resp.rect);
    let trailing = if offer_overlay { OVERLAY_BUTTON } else { 0.0 };
    paint_row(
        ui,
        &resp,
        RowKind::Variable,
        var_info.leaf_name(),
        units,
        is_selected,
        trailing,
    );
    if offer_overlay && overlay_button(ui, resp.rect).clicked() {
        ctx.newly_overlaid_idx = Some(idx);
    }
    let clicked = resp.clicked();
    resp.on_hover_ui(|ui| variable_details(ui, var_info));
    if clicked {
        ctx.newly_selected_idx = Some(idx);
    }
}

/// The hover details of a variable row: full name, group, type, shape and description.
fn variable_details(ui: &mut egui::Ui, var_info: &VariableInfo) {
    ui.label(egui::RichText::new(&var_info.name).strong());
    if let Some(group) = var_info.group_path() {
        ui.horizontal(|ui| {
            ui.label("Group:");
            ui.icon(Icon::Folder, IconSize::Sm);
            for (i, seg) in group.split('/').filter(|s| !s.is_empty()).enumerate() {
                if i > 0 {
                    ui.icon_toned(Icon::ChevronRight, IconSize::Xs, IconTone::Muted);
                }
                ui.label(seg);
            }
        });
    }
    ui.label(format!("Type: [{}]", var_info.data_type));
    ui.label(format!("Shape: {:?}", var_info.shape));
    if let Some(desc) = &var_info.long_name {
        ui.label(format!("Description: {}", desc));
    }
}
