use super::nav::{NodeKey, row_id};
use super::row::{RowKind, allocate_row, paint_row};
use super::tree::VariableTreeContext;
use crate::data::VariableInfo;
use crate::i18n::{VariablesKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::stack_str;

pub const MAX_ITEMS_PER_LEVEL: usize = 100;

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
    truncation_note(ui, total, VariablesKey::TruncatedVariables);
}

/// Muted note under a level listing more than [`MAX_ITEMS_PER_LEVEL`] items;
/// `note` is a template taking the shown and total counts.
pub fn truncation_note(ui: &mut egui::Ui, total: usize, note: VariablesKey) {
    if total <= MAX_ITEMS_PER_LEVEL {
        return;
    }
    let mut buf = [0; 384];
    let text = crate::i18n::fill(&mut buf, tr(note), &[&MAX_ITEMS_PER_LEVEL, &total]);
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
    if let Some(r) = var_info.multiscale.as_ref().filter(|r| r.is_base()) {
        super::stack::render_stack_row(ui, var_info, &r.set, idx, ctx);
        return;
    }
    let is_selected = ctx.selected_idx == idx;
    let units = var_info.units.as_deref().unwrap_or("");

    let resp = allocate_row(ui, row_id(NodeKey::Variable(idx), ctx.search_active));
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
                label_with_colon(ui, VariablesKey::Group);
                ui.icon(Icon::Folder, IconSize::Sm);
                for (i, seg) in group.split('/').filter(|s| !s.is_empty()).enumerate() {
                    if i > 0 {
                        ui.icon_toned(Icon::ChevronRight, IconSize::Xs, IconTone::Muted);
                    }
                    ui.label(seg);
                }
            });
        }
        let mut buf = [0; 160];
        ui.label(stack_str(
            &mut buf,
            format_args!("{}: [{}]", tr(VariablesKey::DataType), var_info.data_type),
        ));
        ui.label(stack_str(
            &mut buf,
            format_args!("{}: {:?}", tr(VariablesKey::Shape), var_info.shape),
        ));
        if let Some(desc) = &var_info.long_name {
            ui.horizontal_wrapped(|ui| {
                label_with_colon(ui, VariablesKey::Description);
                ui.label(desc.as_str());
            });
        }
    });

    if clicked {
        ctx.newly_selected_idx = Some(idx);
        ctx.newly_selected_level = crate::app::layers::LevelChoice::Auto;
    }
}

/// `key`'s text followed by a colon, as a row label.
fn label_with_colon(ui: &mut egui::Ui, key: VariablesKey) {
    let mut buf = [0; 64];
    ui.label(stack_str(&mut buf, format_args!("{}:", tr(key))));
}
