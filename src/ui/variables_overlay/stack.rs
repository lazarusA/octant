//! A multiscale variable's row: one row under the stack's name (clicking it
//! reads the level that fits, Auto), whose chevron folds down one row per
//! level (clicking one pins that level).

use super::nav::{NodeKey, row_id};
use super::row::{RowKind, allocate_row, in_chevron_slot, paint_row};
use super::tree::VariableTreeContext;
use crate::app::layers::LevelChoice;
use crate::data::VariableInfo;
use crate::data::multiscale::Multiscale;
use crate::i18n::{VariablesKey, fill, tr};
use egui::Context;
use egui::collapsing_header::CollapsingState;

/// The row of level 0 variable `idx` of `set`, and its level rows when open.
pub fn render_stack_row(
    ui: &mut egui::Ui,
    var: &VariableInfo,
    set: &Multiscale,
    idx: usize,
    ctx: &mut VariableTreeContext<'_>,
) {
    let mut state = stack_state(ui.ctx(), idx, ctx.search_active);
    let resp = allocate_row(ui, row_id(NodeKey::Variable(idx), ctx.search_active));
    if resp.clicked() {
        let on_chevron = resp
            .interact_pointer_pos()
            .is_some_and(|p| in_chevron_slot(resp.rect, p));
        if on_chevron {
            state.toggle(ui);
            state.store(ui.ctx());
        } else {
            select(ctx, idx, LevelChoice::Auto);
        }
    }
    let selected = ctx.selected_idx == idx && ctx.selected_level == LevelChoice::Auto;
    let kind = RowKind::Stack {
        open: state.is_open(),
    };
    let units = var.units.as_deref().unwrap_or("");
    paint_row(ui, &resp, kind, var.display_name(), units, selected);
    state.show_body_indented(&resp, ui, |ui| {
        for (level, info) in set.levels.iter().enumerate() {
            level_row(ui, idx, level, &info.shape, ctx);
        }
    });
    resp.on_hover_text(var.name.as_str());
}

/// Level `level` (of `shape`) of the stack whose level 0 is variable `idx`.
fn level_row(
    ui: &mut egui::Ui,
    idx: usize,
    level: usize,
    shape: &[u64],
    ctx: &mut VariableTreeContext<'_>,
) {
    let resp = allocate_row(ui, row_id(NodeKey::Level(idx, level), ctx.search_active));
    let choice = LevelChoice::Pinned(level);
    if resp.clicked() {
        select(ctx, idx, choice);
    }
    let selected = ctx.selected_idx == idx && ctx.selected_level == choice;
    let mut name = [0; 64];
    let name = fill(&mut name, tr(VariablesKey::LevelN), &[&level]);
    let mut detail = [0; 96];
    let detail = shape_text(&mut detail, shape);
    paint_row(ui, &resp, RowKind::Level, name, detail, selected);
}

fn select(ctx: &mut VariableTreeContext<'_>, idx: usize, level: LevelChoice) {
    ctx.newly_selected = Some((idx, level));
}

/// `shape` as `a x b x c`, in `buf`.
fn shape_text<'b>(buf: &'b mut [u8], shape: &[u64]) -> &'b str {
    struct Shape<'s>(&'s [u64]);
    impl std::fmt::Display for Shape<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for (i, n) in self.0.iter().enumerate() {
                write!(f, "{}{n}", if i == 0 { "" } else { " x " })?;
            }
            Ok(())
        }
    }
    crate::utils::stack_str::stack_str(buf, format_args!("{}", Shape(shape)))
}

/// Whether a multiscale variable's levels are folded down: closed until
/// opened, while searching too.
pub fn stack_state(ctx: &Context, idx: usize, search_active: bool) -> CollapsingState {
    let id = super::nav::base_id().with(("var_tree_stack", search_active, idx));
    CollapsingState::load_with_default_open(ctx, id, false)
}

/// The levels of variable `idx` when it is level 0 of a multiscale stack.
pub(super) fn stack_levels(variables: &[VariableInfo], idx: usize) -> Option<usize> {
    Some(variables.get(idx)?.stack_base()?.set.len())
}
