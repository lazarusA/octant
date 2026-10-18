//! Keyboard focus as windows and panels open and close: focus returns to
//! the widget that opened one once it closes, and a modal window keeps
//! focus inside it while it is open.

use super::focus::{request, visible};
use egui::{Context, Id, Ui};

fn opener_id(key: Id) -> Id {
    key.with("keyboard_opener")
}

/// Remembers the widget holding keyboard focus as what opened `key` (a panel
/// or window), so focus returns to it when that closes.
pub fn remember_opener(ctx: &Context, key: Id) {
    let opener = visible(ctx).then(|| ctx.memory(|m| m.focused())).flatten();
    ctx.data_mut(|d| match opener {
        Some(id) => {
            d.insert_temp(opener_id(key), id);
        }
        None => d.remove::<Id>(opener_id(key)),
    });
}

/// Focuses the widget that opened `key` again, while the keyboard is in use.
pub fn return_to_opener(ctx: &Context, key: Id) {
    let opener = ctx.data_mut(|d| {
        let opener = d.get_temp::<Id>(opener_id(key));
        d.remove::<Id>(opener_id(key));
        opener
    });
    if let Some(id) = opener.filter(|_| visible(ctx)) {
        request(ctx, id);
    }
}

/// Call every frame with whether the window `key` is open: remembers what
/// held keyboard focus as it opened and focuses that again once it closes.
/// Whether it opened this frame.
pub fn follow_window(ctx: &Context, key: Id, open: bool) -> bool {
    let flag = key.with("keyboard_window_open");
    let was = ctx.data_mut(|d| std::mem::replace(d.get_temp_mut_or_default::<bool>(flag), open));
    match (was, open) {
        (false, true) => remember_opener(ctx, key),
        (true, false) => return_to_opener(ctx, key),
        _ => {}
    }
    open && !was
}

/// Keeps keyboard focus in the modal window drawn in `ui` (and the popups
/// above it); once it `opened`, Tab enters it from the start.
pub fn trap(ui: &Ui, opened: bool) {
    let layer = ui.layer_id();
    ui.ctx().memory_mut(|m| {
        m.set_modal_layer(layer);
        if let Some(id) = m.focused().filter(|_| opened) {
            m.surrender_focus(id);
        }
    });
}
