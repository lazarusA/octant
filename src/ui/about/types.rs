//! Tab state and constants for the About Octant modal dialog.

pub const ICONS_TAB_LABEL: &str = "Vector Icons (46)";

pub const ICON_CATEGORIES: [&str; 6] = [
    "All",
    "Navigation & Menus",
    "Playback & Timeline",
    "Plot Types & Colormaps",
    "Data Store & Files",
    "Tools & Status Badges",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AboutTab {
    Overview,
    Icons,
}
