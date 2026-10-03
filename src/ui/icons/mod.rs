//! Native procedural vector icons for Octant.
//!
//! Provides resolution-independent, theme-adaptive vector icons drawn directly into
//! egui's GPU vertex stream with `egui::Painter`. Replaces font-dependent emojis with
//! crisp scientific icons.

pub mod button;
mod canvas;
mod ext;
mod files;
mod marks;
mod meta;
mod nav;
mod paint;
mod palette;
mod playback;
mod plots;
#[cfg(test)]
mod sheet;
mod status;
mod store;
pub mod style;
#[cfg(test)]
mod tests;

pub use button::{TOOLBAR_ITEM_HEIGHT, ToolbarButton};
pub use ext::UiIconExt;
pub use meta::Icon;
pub use style::{ICON_GAP, IconSize, IconTone};
