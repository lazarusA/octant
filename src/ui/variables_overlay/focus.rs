//! Focus handoff for the overlay: focus the search field when the overlay
//! opens, and claim the arrow keys for tree rows from egui's spatial focus
//! movement so only the tree navigation moves focus.

use egui::{Context, EventFilter, FocusDirection, Id, Key};

/// Rows own every arrow key (and Escape) while focused, so egui's spatial
/// focus movement never competes with tree navigation.
const ROW_FILTER: EventFilter = EventFilter {
    tab: false,
    horizontal_arrows: true,
    vertical_arrows: true,
    escape: true,
};

/// Focus a row. egui only accepts its key filter once the row has held focus
/// for a frame, so repaint at once to claim the keys before the next press.
pub fn focus_row(ctx: &Context, id: Id) {
    ctx.memory_mut(|m| {
        m.request_focus(id);
        m.move_focus(FocusDirection::None);
    });
    ctx.request_repaint();
}

/// Keep the arrow keys claimed while a row holds focus, and cancel egui's
/// spatial move for frames where the filter is not in place yet. Tab is
/// left alone so it still moves focus out of the tree.
pub fn lock_row_keys(ctx: &Context, id: Id) {
    const ARROWS: [Key; 4] = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
    ];
    let arrow = ctx.input(|i| ARROWS.iter().any(|&k| i.key_pressed(k)));
    ctx.memory_mut(|m| {
        m.set_focus_lock_filter(id, ROW_FILTER);
        if arrow {
            m.move_focus(FocusDirection::None);
        }
    });
}

fn pending_id() -> Id {
    Id::new("octant_variables_focus_on_open")
}

fn shown_id() -> Id {
    Id::new("octant_variables_shown_dataset")
}

/// Track which dataset the overlay shows (`None` while closed, loading or
/// empty). Opening the overlay, or a new dataset arriving while it stays open,
/// leaves a pending request to focus the search field, so keys work without a
/// click.
pub fn note_shown(ctx: &Context, dataset: Option<Id>) {
    ctx.data_mut(|d| {
        let previous = d.get_temp::<Id>(shown_id());
        match dataset {
            Some(key) if previous != Some(key) => {
                d.insert_temp(shown_id(), key);
                d.insert_temp(pending_id(), true);
            }
            Some(_) => {}
            None => {
                d.remove::<Id>(shown_id());
                d.remove::<bool>(pending_id());
            }
        }
    });
}

/// Take the pending focus request. It stays pending while the overlay body
/// is collapsed, so the search field is focused once it is first shown.
pub fn take_open_focus(ctx: &Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(pending_id()).unwrap_or(false))
}
