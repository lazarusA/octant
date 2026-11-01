//! Visualization pipeline initialization, aspect scaling, color uniforms, and canvas painting.

pub mod aspect;
pub mod axis_info;
mod coastline_bounds;
pub mod color;
pub(crate) mod greyscale;
mod mesh_grid;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod mesh_grid_tests;
pub mod paint;
mod paint_coastlines;
pub mod paint_layer;
mod paint_tiles;
mod point_grid;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod point_grid_tests;
pub mod rebuild;
mod rebuild_2d;
mod shown;

pub(crate) use shown::needs_pyramid;
