//! The colormap list by keyboard: Down from the search field enters it,
//! Up, Down, Home and End move between rows, scrolling the virtualized list
//! to rows it does not draw; the focused row previews like a hovered one and
//! Enter picks it.

use crate::ui::keyboard::focus;
use egui::{Context, Id, Key};
use std::ops::Range;

/// The row of colormap `colormap`, the same however the list scrolls.
pub fn row_id(colormap: u32) -> Id {
    Id::unique(("colormap_row", colormap))
}

/// The index of `ids` keyboard focus moves to this frame from the row of
/// `focused` (the colormap focused last frame), while that row keeps focus.
pub fn target(ctx: &Context, ids: &[u32], focused: Option<u32>) -> Option<usize> {
    let colormap = focused?;
    let id = row_id(colormap);
    if ctx.memory(|m| m.focused()) != Some(id) {
        return None;
    }
    focus::claim_arrows(ctx, id);
    let at = ids.iter().position(|&i| i == colormap)?;
    let last = ids.len().checked_sub(1)?;
    let keys = [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End];
    match focus::pressed(ctx, keys)? {
        Key::ArrowUp => at.checked_sub(1),
        Key::ArrowDown => (at < last).then_some(at + 1),
        Key::Home => Some(0),
        Key::End => Some(last),
        _ => None,
    }
}

/// The scroll offset bringing row `index` (rows `step` apart, `view` tall
/// list) into view, when it lies outside the rows drawn last frame.
pub fn offset_showing(index: usize, step: f32, view: f32, drawn: &Range<usize>) -> Option<f32> {
    // The first and last drawn rows may be cut by the list's edges.
    let shown = drawn.start + 1..drawn.end.saturating_sub(1);
    if shown.contains(&index) {
        return None;
    }
    let top = index as f32 * step;
    Some(if index < shown.start {
        top
    } else {
        (top + step - view).max(0.0)
    })
}
