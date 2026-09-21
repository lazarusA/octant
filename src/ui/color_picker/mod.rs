//! Flexible custom shape color picker widget and popup subsystem.

pub mod popup;
pub mod shape;
pub mod widget;

#[cfg(test)]
mod tests;

pub use shape::{ColorShape, CustomShapeFn};
pub use widget::ShapeColorPicker;
