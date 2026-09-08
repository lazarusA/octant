//! Modular slicing algorithms and coordinate extraction for N-dimensional tensor blocks.

pub mod common;
pub mod coords;
pub mod slice_2d;
pub mod slice_3d;

pub use coords::extract_sliced_coords_for_dim;
pub use slice_2d::slice_2d_with_ranges;
pub use slice_3d::volume_with_ranges;
