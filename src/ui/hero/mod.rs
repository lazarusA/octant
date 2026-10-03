//! Hero Landing Page and animated intake controls.

pub mod chip_nav;
pub mod chips;
pub mod feedback;
pub mod focus;
pub mod intake;
pub mod landing;
pub mod state;
pub mod style;
pub mod widget;

#[cfg(test)]
mod tests;

pub use landing::show_hero_landing;
pub use state::{HeroState, SEQUENCE};
pub use widget::draw_octant_widget;
