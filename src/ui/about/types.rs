//! Tab state and icon-category helpers for the About Octant dialog.

use crate::ui::icons::Icon;
use crate::utils::stack_str;

/// Category-filter entry that shows every icon.
pub const ALL_CATEGORIES: &str = "All";

/// Number of entries in the category filter: "All" plus each category.
pub fn icon_category_count() -> usize {
    Icon::CATEGORIES.len() + 1
}

/// Label of category-filter entry `idx`; 0 is "All".
pub fn icon_category(idx: usize) -> &'static str {
    idx.checked_sub(1)
        .and_then(|i| Icon::CATEGORIES.get(i))
        .copied()
        .unwrap_or(ALL_CATEGORIES)
}

/// Categories to list for filter entry `idx`: all of them for "All".
pub fn icon_categories_for(idx: usize) -> &'static [&'static str] {
    match idx.checked_sub(1) {
        Some(i) => Icon::CATEGORIES.get(i..=i).unwrap_or(Icon::CATEGORIES),
        None => Icon::CATEGORIES,
    }
}

/// "Vector Icons (N)" with the live icon count, written into `buf`.
pub fn icons_tab_label(buf: &mut [u8; 32]) -> &str {
    stack_str(buf, format_args!("Vector Icons ({})", Icon::ALL.len()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AboutTab {
    Overview,
    Icons,
}
