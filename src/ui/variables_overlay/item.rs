use super::row::{RowKind, allocate_row, paint_row};
use crate::data::VariableInfo;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

pub const MAX_ITEMS_PER_LEVEL: usize = 100;

pub fn render_variable_list(
    ui: &mut egui::Ui,
    indices: &[usize],
    variables: &[VariableInfo],
    selected_idx: usize,
    newly_selected_idx: &mut Option<usize>,
) {
    let total = indices.len();
    for &idx in indices.iter().take(MAX_ITEMS_PER_LEVEL) {
        if let Some(var_info) = variables.get(idx) {
            render_variable_row(ui, var_info, idx, selected_idx, newly_selected_idx);
        }
    }
    if total > MAX_ITEMS_PER_LEVEL {
        ui.label(
            egui::RichText::new(format!(
                "Showing 100 of {} variables in this folder. Use search to discover all.",
                total
            ))
            .small()
            .italics()
            .color(ui.visuals().weak_text_color()),
        );
    }
}

pub fn render_variable_row(
    ui: &mut egui::Ui,
    var_info: &VariableInfo,
    idx: usize,
    selected_idx: usize,
    newly_selected_idx: &mut Option<usize>,
) {
    let is_selected = selected_idx == idx;
    let units = var_info.units.as_deref().unwrap_or("");

    let resp = allocate_row(ui);
    paint_row(
        ui,
        &resp,
        RowKind::Variable,
        var_info.leaf_name(),
        units,
        is_selected,
    );
    let clicked = resp.clicked();

    resp.on_hover_ui(|ui| {
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
    });

    if clicked {
        *newly_selected_idx = Some(idx);
    }
}
