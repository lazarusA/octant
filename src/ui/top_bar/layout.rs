//! Width-driven collapse of top-bar items from icon+label to icon only.

/// Every item drawn in the top bar, in left-to-right order of the left group,
/// followed by the right-aligned group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TopBarItem {
    Brand,
    Dataset,
    Variables,
    Dimensions,
    PlotType,
    Colormap,
    Settings,
    Status,
    Cache,
    Theme,
}

pub(super) const ITEM_COUNT: usize = 10;

/// Items drawn left-aligned after the brand header.
pub(super) const LEFT_ITEMS: [TopBarItem; 6] = [
    TopBarItem::Dataset,
    TopBarItem::Variables,
    TopBarItem::Dimensions,
    TopBarItem::PlotType,
    TopBarItem::Colormap,
    TopBarItem::Settings,
];

/// Order in which items drop their labels as the bar narrows: least
/// important label first, brand text last.
pub(super) const COLLAPSE_ORDER: [TopBarItem; ITEM_COUNT] = [
    TopBarItem::Status,
    TopBarItem::Theme,
    TopBarItem::Cache,
    TopBarItem::Settings,
    TopBarItem::Colormap,
    TopBarItem::PlotType,
    TopBarItem::Dimensions,
    TopBarItem::Variables,
    TopBarItem::Dataset,
    TopBarItem::Brand,
];

/// Horizontal space an item occupies, including its trailing spacing and
/// separators, in both display modes. Absent items have zero widths.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ItemWidths {
    pub full: f32,
    pub compact: f32,
}

/// Per-item compact flags, indexed by `TopBarItem as usize`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CompactFlags([bool; ITEM_COUNT]);

impl CompactFlags {
    #[inline]
    pub fn get(self, item: TopBarItem) -> bool {
        self.0[item as usize]
    }
}

/// Collapse items in [`COLLAPSE_ORDER`] until the bar fits `available` width.
///
/// If the bar does not fit even with every item collapsed, all items are
/// compact and the remainder is clipped by the panel.
pub(super) fn compute_compact(widths: &[ItemWidths; ITEM_COUNT], available: f32) -> CompactFlags {
    let mut flags = [false; ITEM_COUNT];
    let mut total: f32 = widths.iter().map(|w| w.full).sum();

    for item in COLLAPSE_ORDER {
        if total <= available {
            break;
        }
        let w = widths[item as usize];
        flags[item as usize] = true;
        total -= w.full - w.compact;
    }
    CompactFlags(flags)
}
