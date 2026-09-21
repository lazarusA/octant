use crate::data::VariableInfo;
use crate::ui::icons::{Icon, UiIconExt};

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
    let leaf_name = var_info.leaf_name();

    let row_resp = ui.horizontal(|ui| {
        ui.icon(Icon::VariableDoc, 11.0);
        if let Some(units) = &var_info.units {
            if !units.is_empty() {
                let mut buf = [0u8; 96];
                let label_text = format_leaf_units(&mut buf, leaf_name, units);
                ui.selectable_label(is_selected, egui::RichText::new(label_text).strong())
            } else {
                ui.selectable_label(is_selected, egui::RichText::new(leaf_name).strong())
            }
        } else {
            ui.selectable_label(is_selected, egui::RichText::new(leaf_name).strong())
        }
    });

    let clicked = row_resp.response.clicked() || row_resp.inner.clicked();

    row_resp.response.on_hover_ui(|ui| {
        ui.label(egui::RichText::new(&var_info.name).strong());
        if let Some(group) = var_info.group_path() {
            ui.horizontal(|ui| {
                ui.label("Group:");
                ui.icon(Icon::Folder, 11.0);
                for (i, seg) in group.split('/').filter(|s| !s.is_empty()).enumerate() {
                    if i > 0 {
                        ui.icon_colored(Icon::ChevronRight, 8.0, ui.visuals().weak_text_color());
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

fn format_leaf_units<'a>(buf: &'a mut [u8; 96], leaf: &str, units: &str) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{}  ({})", leaf, units);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
