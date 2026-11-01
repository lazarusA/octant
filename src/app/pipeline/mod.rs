//! Visualization pipeline initialization, aspect scaling, color uniforms, and canvas painting.

pub mod aspect;
pub mod axis_info;
mod coastline_bounds;
pub mod color;
pub(crate) mod greyscale;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod mesh_grid_tests;
pub mod paint;
pub mod paint_layer;
pub mod rebuild;
mod rebuild_2d;

pub(crate) use rebuild_2d::needs_pyramid;
