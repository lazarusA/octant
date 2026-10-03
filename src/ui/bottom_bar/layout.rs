//! Bottom-bar items, their placement, and the order they collapse in.

use crate::ui::toolbar::{BarItem, ItemWidths};

/// Every item in the expanded bar. The timeline slider is not an item: it
/// stretches between the left and right groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BottomBarItem {
    First,
    Prev,
    PlayPause,
    Next,
    Last,
    Loop,
    DateInfo,
    StartBadge,
    StepSize,
    EndBadge,
    Crop,
    Fps,
    Save,
}

pub(super) const ITEM_COUNT: usize = 13;

/// Items left of the slider, in drawing order.
pub(super) const LEFT_ITEMS: [BottomBarItem; 8] = [
    BottomBarItem::First,
    BottomBarItem::Prev,
    BottomBarItem::PlayPause,
    BottomBarItem::Next,
    BottomBarItem::Last,
    BottomBarItem::Loop,
    BottomBarItem::DateInfo,
    BottomBarItem::StartBadge,
];

/// Items right of the slider, in drawing order.
pub(super) const RIGHT_ITEMS: [BottomBarItem; 5] = [
    BottomBarItem::StepSize,
    BottomBarItem::EndBadge,
    BottomBarItem::Crop,
    BottomBarItem::Fps,
    BottomBarItem::Save,
];

/// Collapse priority as the bar narrows: data badges hide first, then the
/// date shrinks to its icon (its popover keeps the details), then buttons
/// drop their labels.
/// First / previous / next / last are always icon-only, so they are absent.
pub(super) const COLLAPSE_ORDER: [BottomBarItem; 9] = [
    BottomBarItem::StepSize,
    BottomBarItem::StartBadge,
    BottomBarItem::EndBadge,
    BottomBarItem::DateInfo,
    BottomBarItem::Fps,
    BottomBarItem::Loop,
    BottomBarItem::Crop,
    BottomBarItem::Save,
    BottomBarItem::PlayPause,
];

/// Narrowest the timeline slider gets before items start collapsing.
pub(super) const SLIDER_MIN_W: f32 = 80.0;

impl BarItem for BottomBarItem {
    fn index(self) -> usize {
        self as usize
    }
}

impl BottomBarItem {
    /// Data badges have nothing to show as an icon, so they hide when
    /// collapsed (their compact width is zero).
    pub fn hides_when_compact(self) -> bool {
        matches!(
            self,
            BottomBarItem::StartBadge | BottomBarItem::StepSize | BottomBarItem::EndBadge
        )
    }
}

/// Per-item compact flags for the bottom bar.
pub(super) type CompactFlags = crate::ui::toolbar::CompactFlags<ITEM_COUNT>;

/// Collapse bottom-bar items in [`COLLAPSE_ORDER`] until they fit `available`.
pub(super) fn compute_compact(widths: &[ItemWidths; ITEM_COUNT], available: f32) -> CompactFlags {
    crate::ui::toolbar::compute_compact(widths, &COLLAPSE_ORDER, available)
}
