//! Dynamic plot axis renderer with auto-attaching canvas borders and inward/outward ticks.

mod draw;
mod segments;
#[cfg(test)]
mod tests;
mod ticks;
mod title;

pub use draw::{PlotAxisOptions, draw_plot_axes};
pub use ticks::TickMark;
