//! Width-driven collapse shared by the top and bottom bars: items drop their
//! labels (or, for data-only items, disappear) in a fixed priority order until
//! the bar fits, instead of moving into an overflow menu.

#[cfg(test)]
mod tests;

/// Width `ui.separator()` takes in a horizontal bar (its default `spacing`),
/// excluding item spacing.
pub const SEPARATOR_WIDTH: f32 = 6.0;

/// Horizontal space an item occupies, including its trailing spacing and
/// separators, in both display modes. Absent items have zero widths; items
/// that hide when collapsed have a zero `compact` width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemWidths {
    pub full: f32,
    pub compact: f32,
}

impl ItemWidths {
    /// An item whose width does not change when collapsed (e.g. icon-only).
    pub const fn fixed(width: f32) -> Self {
        Self {
            full: width,
            compact: width,
        }
    }
}

/// A bar item enum whose variants index the bar's width table.
pub trait BarItem: Copy {
    fn index(self) -> usize;
}

/// Per-item compact flags for a bar with `N` items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactFlags<const N: usize>([bool; N]);

impl<const N: usize> CompactFlags<N> {
    #[inline]
    pub fn get(self, item: impl BarItem) -> bool {
        self.0.get(item.index()).copied().unwrap_or(false)
    }
}

/// Collapse items in `order` until the total width fits `available`.
///
/// If the bar does not fit even with every listed item collapsed, all of them
/// are compact and the remainder is clipped by the panel.
pub fn compute_compact<T: BarItem, const N: usize>(
    widths: &[ItemWidths; N],
    order: &[T],
    available: f32,
) -> CompactFlags<N> {
    let mut flags = [false; N];
    let mut total: f32 = widths.iter().map(|w| w.full).sum();

    for &item in order {
        if total <= available {
            break;
        }
        let Some(w) = widths.get(item.index()) else {
            continue;
        };
        flags[item.index()] = true;
        total -= w.full - w.compact;
    }
    CompactFlags(flags)
}
