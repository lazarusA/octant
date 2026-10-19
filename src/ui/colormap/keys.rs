//! The colormap list by keyboard: Down from the search field enters it,
//! Up, Down, Home and End move between rows, scrolling the virtualized list
//! to rows it does not draw, and Up on the first row returns to the search
//! field; the focused row previews like a hovered one and Enter picks it.

use crate::ui::keyboard::focus;
use crate::ui::keyboard::list_nav::Above;
use egui::{Context, Id, Key, ScrollArea};
use std::ops::Range;

/// The row of colormap `colormap`, the same however the list scrolls.
pub fn row_id(colormap: u32) -> Id {
    Id::unique(("colormap_row", colormap))
}

/// The list's rows as the keys see them.
pub struct Rows<'a> {
    /// Every matching colormap, in list order.
    pub ids: &'a [u32],
    /// The rows drawn last frame.
    pub drawn: Range<usize>,
    /// Distance between row tops.
    pub step: f32,
    /// Height of the list.
    pub view: f32,
}

impl Rows<'_> {
    /// Focuses the row the keys move to this frame (from the row of
    /// `focused`, the colormap focused last frame, or the first row when Down
    /// was pressed in the search field `above`), scrolling `area` to it when
    /// it was not drawn.
    pub fn follow_keys(
        &self,
        ctx: &Context,
        area: ScrollArea,
        focused: Option<u32>,
        above: Above,
    ) -> ScrollArea {
        let enter = above.enter.then_some(0);
        let Some(index) = target(ctx, self.ids, focused, above.id).or(enter) else {
            return area;
        };
        if let Some(&id) = self.ids.get(index) {
            focus::request(ctx, row_id(id));
        }
        match offset_showing(index, self.step, self.view, &self.drawn) {
            Some(offset) => area.vertical_scroll_offset(offset),
            None => area,
        }
    }
}

/// The index of `ids` keyboard focus moves to this frame from the row of
/// `focused` (the colormap focused last frame), while that row keeps focus;
/// Up on the first row focuses the search field `search` instead.
fn target(ctx: &Context, ids: &[u32], focused: Option<u32>, search: Id) -> Option<usize> {
    let colormap = focused?;
    let id = row_id(colormap);
    if ctx.memory(|m| m.focused()) != Some(id) {
        return None;
    }
    focus::claim_arrows(ctx, id);
    let keys = [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End];
    let key = focus::pressed(ctx, keys)?;
    let at = ids.iter().position(|&i| i == colormap)?;
    let next = focus::step_index(key, at, ids.len());
    if next.is_none() && key == Key::ArrowUp {
        focus::request(ctx, search);
    }
    next
}

/// The scroll offset bringing row `index` (rows `step` apart, `view` tall
/// list) into view, when it lies outside the rows drawn last frame.
fn offset_showing(index: usize, step: f32, view: f32, drawn: &Range<usize>) -> Option<f32> {
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
