//! Search field focus for the overlay: focused when the overlay opens or a
//! new dataset arrives while it stays open.

use egui::{Context, Id};

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
