//! Modular slicing algorithms and coordinate extraction for N-dimensional tensor blocks.

pub mod common;
pub mod composite;
pub mod coords;
pub mod copy;
pub mod slice_1d;
pub mod slice_2d;
pub mod slice_3d;

pub use composite::{
    ChannelColorConfig, DEFAULT_CHANNEL_COLORS, parse_hex_color, slice_cmyk_composite,
    slice_multichannel_composite_nd, slice_rgb_composite, slice_rgb_composite_nd,
};
pub use coords::extract_sliced_coords_for_dim;
pub use slice_2d::slice_2d_with_ranges;
pub use slice_3d::volume_with_ranges;
