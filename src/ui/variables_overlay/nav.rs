//! Keyboard navigation for the variable tree (WAI-ARIA tree view pattern).
//!
//! Rows have stable ids derived from [`NodeKey`], so focus can be moved to a
//! specific row. The visible row list is only built on frames where a
//! navigation key is pressed; idle frames allocate nothing.

pub use super::focus::{focus_row, lock_row_keys};
use super::item::MAX_ITEMS_PER_LEVEL;
use super::tree::VariableTreeContext;
use crate::data::VariableTreeGroup;
use egui::collapsing_header::CollapsingState;
use egui::{Context, Id, Key, Modifiers};

/// Identity of a tree row: a folder by its full path (`""` for the root
/// variables folder) or a variable by its index into the dataset variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKey<'a> {
    Folder(&'a str),
    Variable(usize),
}

fn base_id() -> Id {
    Id::new("octant_variables_tree")
}

/// Open/closed state id of a folder. Search keeps its own state, so results
/// always start expanded and clearing the search restores the browse state.
pub fn folder_id(full_path: &str, search_active: bool) -> Id {
    base_id().with(("var_tree_group", search_active, full_path))
}

pub fn row_id(key: NodeKey<'_>, search_active: bool) -> Id {
    base_id().with(("var_tree_row", search_active, key))
}

pub fn is_folder_open(ctx: &Context, full_path: &str, search_active: bool) -> bool {
    CollapsingState::load(ctx, folder_id(full_path, search_active))
        .map_or(search_active, |s| s.is_open())
}

/// Where focus should go after a key in the search field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchJump {
    /// Down arrow: the first visible row.
    FirstRow,
    /// Enter: the first visible variable, selecting it if it is the only match.
    FirstVariable,
}

#[derive(Clone, Copy)]
struct Row<'a> {
    key: NodeKey<'a>,
    parent: Option<usize>,
    open: Option<bool>,
}

/// Visible rows in display order, mirroring `tree::render_tree_group`.
fn visible_rows<'a>(ctx: &Context, root: &'a VariableTreeGroup, search: bool) -> Vec<Row<'a>> {
    let mut rows = Vec::new();
    let vars = |rows: &mut Vec<Row<'a>>, indices: &[usize], parent| {
        for &idx in indices.iter().take(MAX_ITEMS_PER_LEVEL) {
            let key = NodeKey::Variable(idx);
            rows.push(Row {
                key,
                parent,
                open: None,
            });
        }
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

fn set_open(ctx: &Context, key: NodeKey<'_>, search: bool, open: bool) {
    if let NodeKey::Folder(path) = key {
        let mut state =
            CollapsingState::load_with_default_open(ctx, folder_id(path, search), search);
        state.set_open(open);
        state.store(ctx);
        ctx.request_repaint();
    }
}

const NAV_KEYS: [Key; 7] = [
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::Home,
    Key::End,
    Key::Escape,
];

/// Handle tree navigation keys. Call before drawing the tree so opened and
/// closed folders draw in their new state on the same frame.
pub fn handle_keys(ui: &egui::Ui, root: &VariableTreeGroup, tree: &mut VariableTreeContext<'_>) {
    let ctx = ui.ctx();
    let search = tree.search_active;
    if let Some(jump) = tree.search_jump.take() {
        jump_from_search(ctx, root, tree, jump);
        return;
    }

    let pressed = |k| ui.input(|i| i.modifiers.is_none() && i.key_pressed(k));
    let Some(key) = NAV_KEYS.into_iter().find(|&k| pressed(k)) else {
        return;
    };
    // A row focused only since last frame has no filter yet, so egui itself
    // drops its focus on Escape; find it by last frame's focus instead.
    let focused = ctx.memory(|m| m.focused());
    if focused.is_none() && key != Key::Escape {
        return;
    }

    let rows = visible_rows(ctx, root, search);
    let is_focused = |id| match focused {
        Some(f) => f == id,
        None => ctx.memory(|m| m.had_focus_last_frame(id)),
    };
    let Some(at) = rows.iter().position(|r| is_focused(row_id(r.key, search))) else {
        return;
    };
    apply_key(ctx, &rows, at, key, search, tree.search_id);
}

/// Act on `key` for the focused row `rows[at]`.
fn apply_key(
    ctx: &Context,
    rows: &[Row<'_>],
    at: usize,
    key: Key,
    search: bool,
    search_id: Option<Id>,
) {
    let row = rows[at];
    let focus = |i: usize| focus_row(ctx, row_id(rows[i].key, search));
    let to_search = || {
        if let Some(id) = search_id {
            ctx.memory_mut(|m| m.request_focus(id));
        }
    };

    match key {
        Key::ArrowDown if at + 1 < rows.len() => focus(at + 1),
        Key::ArrowUp if at > 0 => focus(at - 1),
        Key::ArrowUp | Key::Escape => to_search(),
        Key::Home => focus(0),
        Key::End => focus(rows.len() - 1),
        Key::ArrowRight => match row.open {
            Some(false) => set_open(ctx, row.key, search, true),
            Some(true) if rows.get(at + 1).is_some_and(|r| r.parent == Some(at)) => focus(at + 1),
            _ => {}
        },
        Key::ArrowLeft => match (row.open, row.parent) {
            (Some(true), _) => set_open(ctx, row.key, search, false),
            (_, Some(parent)) => focus(parent),
            _ => {}
        },
        _ => {}
    }
}

fn jump_from_search(
    ctx: &Context,
    root: &VariableTreeGroup,
    tree: &mut VariableTreeContext<'_>,
    jump: SearchJump,
) {
    let search = tree.search_active;
    let rows = visible_rows(ctx, root, search);
    let target = match jump {
        SearchJump::FirstRow => rows.first(),
        SearchJump::FirstVariable => rows.iter().find(|r| matches!(r.key, NodeKey::Variable(_))),
    };
    let Some(row) = target else {
        return;
    };
    focus_row(ctx, row_id(row.key, search));
    // The Enter that ended the search edit must not also click the newly focused row.
    ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    if let NodeKey::Variable(idx) = row.key
        && jump == SearchJump::FirstVariable
        && search
        && root.total_variable_count() == 1
    {
        tree.newly_selected_idx = Some(idx);
    }
}
