//! Hero Landing Page and animated intake controls.

pub mod chips;
pub mod feedback;
pub mod intake;
pub mod landing;
pub mod state;
pub mod widget;

pub use landing::show_hero_landing;
pub use state::{HeroState, SEQUENCE};
pub use widget::draw_octant_widget;
