//! Keyboard navigation for the variable tree (WAI-ARIA tree view pattern).
//!
//! Rows have stable ids derived from [`NodeKey`], so focus can be moved to a
//! specific row. The visible row list is only built on frames where a
//! navigation key is pressed; idle frames allocate nothing.

use super::rows::{Row, visible_rows};
use super::stack::stack_state;
use super::tree::VariableTreeContext;
use crate::data::VariableTreeGroup;
use crate::ui::keyboard::focus as key_focus;
use egui::collapsing_header::CollapsingState;
use egui::{Context, Id, Key, Modifiers};

/// Identity of a tree row: a folder by its full path (`""` for the root
/// variables folder) or a variable by its index into the dataset variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKey<'a> {
    Folder(&'a str),
    Variable(usize),
    /// Level `.1` of the multiscale variable whose level 0 is variable `.0`.
    Level(usize, usize),
}

pub(super) fn base_id() -> Id {
    Id::unique("octant_variables_tree")
}

/// Open/closed state id of a folder. Search keeps its own state, so results
/// always start expanded and clearing the search restores the browse state.
pub fn folder_id(full_path: &str, search_active: bool) -> Id {
    base_id().with(("var_tree_group", search_active, full_path))
}

pub fn row_id(key: NodeKey<'_>, search_active: bool) -> Id {
    base_id().with(("var_tree_row", search_active, key))
}

/// A folder's open/closed state. The single place that says folders start
/// open while searching and closed while browsing.
pub fn folder_state(ctx: &Context, full_path: &str, search_active: bool) -> CollapsingState {
    let id = folder_id(full_path, search_active);
    CollapsingState::load_with_default_open(ctx, id, search_active)
}

pub(super) fn is_folder_open(ctx: &Context, full_path: &str, search_active: bool) -> bool {
    folder_state(ctx, full_path, search_active).is_open()
}

/// Where focus should go after a key in the search field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchJump {
    /// Down arrow: the first visible row.
    FirstRow,
    /// Enter: the first visible variable, selecting it if it is the only match.
    FirstVariable,
}

fn set_open(ctx: &Context, key: NodeKey<'_>, search: bool, open: bool) {
    let mut state = match key {
        NodeKey::Folder(path) => folder_state(ctx, path, search),
        NodeKey::Variable(idx) => stack_state(ctx, idx, search),
        NodeKey::Level(..) => return,
    };
    state.set_open(open);
    state.store(ctx);
    ctx.request_repaint();
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

    let Some(key) = key_focus::pressed(ctx, NAV_KEYS) else {
        return;
    };
    // A row focused only since last frame has no filter yet, so egui itself
    // drops its focus on Escape; find it by last frame's focus instead.
    let focused = ctx.memory(|m| m.focused());
    if focused.is_none() && key != Key::Escape {
        return;
    }

    let rows = visible_rows(ctx, root, tree.variables, search);
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
fn apply_key(ctx: &Context, rows: &[Row<'_>], at: usize, key: Key, search: bool, search_id: Id) {
    let row = rows[at];
    let focus = |i: usize| key_focus::request(ctx, row_id(rows[i].key, search));
    // Through `key_focus` so egui's own spatial Up can't carry focus past the field.
    let to_search = || key_focus::request(ctx, search_id);

    match key {
        Key::Escape => to_search(),
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
        // Up, Down, Home and End; Up on the first row returns to the field.
        _ => match key_focus::step_index(key, at, rows.len()) {
            Some(i) => focus(i),
            None if key == Key::ArrowUp => to_search(),
            None => {}
        },
    }
}

fn jump_from_search(
    ctx: &Context,
    root: &VariableTreeGroup,
    tree: &mut VariableTreeContext<'_>,
    jump: SearchJump,
) {
    let search = tree.search_active;
    let rows = visible_rows(ctx, root, tree.variables, search);
    let target = match jump {
        SearchJump::FirstRow => rows.first(),
        SearchJump::FirstVariable => rows.iter().find(|r| matches!(r.key, NodeKey::Variable(_))),
    };
    let Some(row) = target else {
        return;
    };
    key_focus::request(ctx, row_id(row.key, search));
    // The Enter that ended the search edit must not also click the newly focused row.
    ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    if let NodeKey::Variable(idx) = row.key
        && jump == SearchJump::FirstVariable
        && search
        && root.total_variable_count() == 1
    {
        tree.newly_selected = Some((idx, crate::app::layers::LevelChoice::Auto));
    }
}
