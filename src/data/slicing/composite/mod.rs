//! RGB, CMYK, and Multi-Channel bioimaging composite slicing to 24-bit TrueColor `MatrixData`.

pub mod cmyk;
pub mod multichannel;
pub mod rgb;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

pub use cmyk::slice_cmyk_composite;
pub use multichannel::slice_multichannel_composite_nd;
pub use rgb::{slice_rgb_composite, slice_rgb_composite_nd};
pub use types::{ChannelColorConfig, DEFAULT_CHANNEL_COLORS, parse_hex_color};
pub use utils::{compute_normalization_scale, linear_to_srgb, pack_rgb, unpack_rgb};
