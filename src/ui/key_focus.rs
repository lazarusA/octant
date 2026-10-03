//! Keyboard focus for custom arrow-key navigation (variable tree rows, hero
//! sample chips). egui moves focus spatially on arrow keys; widgets that
//! navigate themselves claim the arrows so only their own logic moves focus.

use egui::{Context, EventFilter, FocusDirection, Id, Key};

/// While focused, the widget owns every arrow key and Escape; Tab still moves
/// focus on as usual.
const ARROW_FILTER: EventFilter = EventFilter {
    tab: false,
    horizontal_arrows: true,
    vertical_arrows: true,
    escape: true,
};

const ARROWS: [Key; 4] = [
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
];

/// Focus `id` from keyboard navigation (or a click). egui only accepts the
/// key filter once the widget has held focus for a frame, so repaint at once
/// to claim the keys before the next press.
pub fn request(ctx: &Context, id: Id) {
    ctx.memory_mut(|m| {
        m.request_focus(id);
        m.move_focus(FocusDirection::None);
    });
    ctx.request_repaint();
}

/// Call every frame the widget `id` holds focus: keeps the arrow keys
/// claimed, and cancels egui's spatial move on frames where the filter is
/// not in place yet.
pub fn claim_arrows(ctx: &Context, id: Id) {
    let arrow = ctx.input(|i| ARROWS.iter().any(|&k| i.key_pressed(k)));
    ctx.memory_mut(|m| {
        m.set_focus_lock_filter(id, ARROW_FILTER);
        if arrow {
            m.move_focus(FocusDirection::None);
        }
    });
}

/// The first of `keys` pressed this frame without modifiers.
pub fn pressed<const N: usize>(ctx: &Context, keys: [Key; N]) -> Option<Key> {
    ctx.input(|i| {
        if !i.modifiers.is_none() {
            return None;
        }
        keys.into_iter().find(|&k| i.key_pressed(k))
    })
}
