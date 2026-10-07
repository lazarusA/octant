//! RGB, CMYK, and Multi-Channel bioimaging composite slicing to 24-bit TrueColor `MatrixData`.

pub mod cmyk;
pub mod multichannel;
pub mod probe;
pub mod rgb;
pub mod types;
pub mod utils;
pub mod volume_multichannel;
pub mod volume_rgb;

#[cfg(test)]
mod tests;

pub use cmyk::slice_cmyk_composite;
pub use multichannel::slice_multichannel_composite_nd;
pub use probe::CompositeProbe;
pub use rgb::{slice_rgb_composite, slice_rgb_composite_nd};
pub use types::{ChannelColorConfig, DEFAULT_CHANNEL_COLORS, parse_hex_color};
pub use utils::{
    compute_channel_normalization, compute_normalization_scale, linear_to_srgb,
    normalize_channel_value, pack_rgb, unpack_rgb,
};
pub use volume_multichannel::slice_multichannel_volume_composite_nd;
pub use volume_rgb::slice_rgb_volume_composite_nd;
