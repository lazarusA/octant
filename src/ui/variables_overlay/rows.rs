//! The rows the variable tree shows, in display order, for keyboard
//! navigation (`nav.rs`): folders, variables and the levels of open stacks.

use super::item::MAX_ITEMS_PER_LEVEL;
#[cfg(test)]
use super::nav::row_id;
use super::nav::{NodeKey, is_folder_open};
use super::stack::{stack_levels, stack_state};
use crate::data::{VariableInfo, VariableTreeGroup};
use egui::Context;
#[cfg(test)]
use egui::Id;

#[derive(Clone, Copy)]
pub(super) struct Row<'a> {
    pub key: NodeKey<'a>,
    pub parent: Option<usize>,
    pub open: Option<bool>,
}

/// Visible rows in display order, mirroring `tree::render_tree_group`.
pub(super) fn visible_rows<'a>(
    ctx: &Context,
    root: &'a VariableTreeGroup,
    variables: &[VariableInfo],
    search: bool,
) -> Vec<Row<'a>> {
    let mut rows = Vec::new();
    let vars = |rows: &mut Vec<Row<'a>>, indices: &[usize], parent| {
        push_variable_rows(ctx, rows, indices, parent, variables, search);
    };
    if !root.variable_indices.is_empty() {
        if root.subgroups.is_empty() {
            vars(&mut rows, &root.variable_indices, None);
        } else {
            let open = is_folder_open(ctx, "", search);
            rows.push(Row {
                key: NodeKey::Folder(""),
                parent: None,
                open: Some(open),
            });
            if open {
                let me = Some(rows.len() - 1);
                vars(&mut rows, &root.variable_indices, me);
            }
        }
    }
    let mut stack: Vec<(&VariableTreeGroup, Option<usize>)> = Vec::new();
    let push_groups = |stack: &mut Vec<_>, groups: &'a [VariableTreeGroup], parent| {
        let shown = &groups[..groups.len().min(MAX_ITEMS_PER_LEVEL)];
        stack.extend(shown.iter().rev().map(|g| (g, parent)));
    };
    push_groups(&mut stack, &root.subgroups, None);
    while let Some((group, parent)) = stack.pop() {
        let open = is_folder_open(ctx, &group.full_path, search);
        let key = NodeKey::Folder(&group.full_path);
        rows.push(Row {
            key,
            parent,
            open: Some(open),
        });
        if open {
            let me = Some(rows.len() - 1);
            vars(&mut rows, &group.variable_indices, me);
            push_groups(&mut stack, &group.subgroups, me);
        }
    }
    rows
}

/// The rows of variables `indices` under `parent`: each variable, and
/// under an open multiscale stack one row per level.
fn push_variable_rows(
    ctx: &Context,
    rows: &mut Vec<Row<'_>>,
    indices: &[usize],
    parent: Option<usize>,
    variables: &[VariableInfo],
    search: bool,
) {
    for &idx in indices.iter().take(MAX_ITEMS_PER_LEVEL) {
        let levels = stack_levels(variables, idx);
        let open = levels.map(|_| stack_state(ctx, idx, search).is_open());
        rows.push(Row {
            key: NodeKey::Variable(idx),
            parent,
            open,
        });
        if let (Some(true), Some(levels)) = (open, levels) {
            let me = Some(rows.len() - 1);
            rows.extend((0..levels).map(|level| Row {
                key: NodeKey::Level(idx, level),
                parent: me,
                open: None,
            }));
        }
    }
}

/// Row ids in navigation order, for checking it against the drawn order.
#[cfg(test)]
pub fn visible_row_ids(
    ctx: &Context,
    root: &VariableTreeGroup,
    variables: &[VariableInfo],
    search: bool,
) -> Vec<Id> {
    let rows = visible_rows(ctx, root, variables, search);
    rows.iter().map(|r| row_id(r.key, search)).collect()
}
