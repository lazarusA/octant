//! Native procedural vector icons for Octant.
//!
//! Provides resolution-independent, theme-adaptive vector icons drawn directly into
//! egui's GPU vertex stream with `egui::Painter`. Replaces font-dependent emojis with
//! crisp scientific icons.

pub mod button;
mod ext;
pub mod files;
mod meta;
pub mod nav;
mod paint;
pub mod palette;
pub mod playback;
pub mod plots;
pub mod status;
pub mod store;
pub mod style;
#[cfg(test)]
mod tests;

pub use button::{TOOLBAR_ITEM_HEIGHT, ToolbarButton};
pub use ext::UiIconExt;
pub use meta::Icon;
pub use style::{ICON_GAP, IconSize, IconTone};

use egui::{Pos2, Rect, pos2};

/// Helper to map (0..24) normalized grid coordinates into the target bounding `rect`.
#[inline]
pub(crate) fn grid_p(rect: Rect, gx: f32, gy: f32) -> Pos2 {
    pos2(
        rect.min.x + (gx / 24.0) * rect.width(),
        rect.min.y + (gy / 24.0) * rect.height(),
    )
}
