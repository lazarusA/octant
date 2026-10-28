//! Dynamic plot axis renderer with auto-attaching canvas borders and inward/outward ticks.

mod draw;
mod lines;
mod minor;
#[cfg(test)]
mod minor_tests;
mod segments;
mod steps;
mod style;
#[cfg(test)]
mod tests;
mod ticks;
mod title;
mod visible;

pub use draw::{PlotAxisOptions, draw_plot_axes};
pub use minor::{CellTicks, Lattice};
pub use style::{AxisStyle, MinorPlacement};
pub use ticks::TickMark;
