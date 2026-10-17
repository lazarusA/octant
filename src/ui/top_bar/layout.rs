//! Top-bar items and the order they collapse from icon+label to icon only.

use crate::ui::toolbar::{BarItem, ItemWidths};

/// Every item drawn in the top bar, in left-to-right order of the left group,
/// followed by the right-aligned group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TopBarItem {
    Brand,
    Dataset,
    Variables,
    Dimensions,
    PlotType,
    Settings,
    Status,
    Cache,
    Language,
    Theme,
}

pub(super) const ITEM_COUNT: usize = 10;

/// Items drawn left-aligned after the brand header.
pub(super) const LEFT_ITEMS: [TopBarItem; 5] = [
    TopBarItem::Dataset,
    TopBarItem::Variables,
    TopBarItem::Dimensions,
    TopBarItem::PlotType,
    TopBarItem::Settings,
];

/// Order in which items drop their labels as the bar narrows: least
/// important label first, brand text last.
pub(super) const COLLAPSE_ORDER: [TopBarItem; ITEM_COUNT] = [
    TopBarItem::Status,
    TopBarItem::Language,
    TopBarItem::Theme,
    TopBarItem::Cache,
    TopBarItem::Settings,
    TopBarItem::PlotType,
    TopBarItem::Dimensions,
    TopBarItem::Variables,
    TopBarItem::Dataset,
    TopBarItem::Brand,
];

impl BarItem for TopBarItem {
    fn index(self) -> usize {
        self as usize
    }
}

/// Per-item compact flags for the top bar.
pub(super) type CompactFlags = crate::ui::toolbar::CompactFlags<ITEM_COUNT>;

/// Collapse top-bar items in [`COLLAPSE_ORDER`] until they fit `available`.
pub(super) fn compute_compact(widths: &[ItemWidths; ITEM_COUNT], available: f32) -> CompactFlags {
    crate::ui::toolbar::compute_compact(widths, &COLLAPSE_ORDER, available)
}
