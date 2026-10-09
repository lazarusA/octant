use super::nav::{NodeKey, row_id};
use super::row::{RowKind, Trailing, allocate_row, paint_row};
use super::tree::VariableTreeContext;
use crate::data::VariableInfo;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

pub const MAX_ITEMS_PER_LEVEL: usize = 100;
/// Side of the "Add as overlay" button.
const OVERLAY_BUTTON: f32 = 20.0;
/// Space between the button and the row's right edge.
const OVERLAY_PAD: f32 = 8.0;
/// Room the button and its padding leave free at the row's right end.
const OVERLAY_ROOM: f32 = OVERLAY_BUTTON + OVERLAY_PAD + 4.0;

/// The "Add as overlay" button's square at the right end of `row`.
fn overlay_button_rect(row: egui::Rect) -> egui::Rect {
    let side = row.height().min(OVERLAY_BUTTON);
    let center = egui::pos2(row.right() - OVERLAY_PAD - side * 0.5, row.center().y);
    egui::Rect::from_center_size(center, egui::vec2(side, side))
}

/// The "Add as overlay" button in `rect`, under `id`: an accent tint and
/// icon while hovered, independent of the row's own highlight.
fn overlay_button(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui
        .interact(rect, id, egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let visuals = ui.visuals();
    let tone = if response.hovered() {
        let fill = IconTone::Accent.themed_tint(visuals, 40, 32);
        let radius = visuals.widgets.hovered.corner_radius;
        ui.painter().rect_filled(rect, radius, fill);
        IconTone::Accent
    } else {
        IconTone::Default
    };
    let icon = egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(IconSize::Sm.px()));
    Icon::Layers.paint(ui.painter(), icon, tone.color(visuals), visuals.dark_mode);
    response.on_hover_text("Add as overlay")
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

    let id = row_id(NodeKey::Variable(idx), ctx.search_active);
    let resp = allocate_row(ui, id);
    // Shown while the pointer is anywhere on the row, button included.
    let offer_overlay = ctx.can_overlay && ui.rect_contains_pointer(resp.rect);
    let button = overlay_button_rect(resp.rect);
    let trailing = Trailing {
        width: if offer_overlay { OVERLAY_ROOM } else { 0.0 },
        hovered: offer_overlay && ui.rect_contains_pointer(button),
    };
    paint_row(
        ui,
        &resp,
        RowKind::Variable,
        var_info.leaf_name(),
        units,
        is_selected,
        trailing,
    );
    if offer_overlay && overlay_button(ui, button, id.with("overlay")).clicked() {
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
