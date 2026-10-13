//! Types and configuration structures for multi-channel color compositing.

use serde::{Deserialize, Serialize};

pub use crate::data::ome::ChannelFamily;

/// High-contrast default color cycle for multi-channel bioimaging overlay (napari standard).
pub const DEFAULT_CHANNEL_COLORS: [[u8; 3]; 8] = [
    [0, 255, 0],   // Green
    [255, 0, 255], // Magenta
    [0, 255, 255], // Cyan
    [255, 255, 0], // Yellow
    [255, 0, 0],   // Red
    [0, 0, 255],   // Blue
    [255, 128, 0], // Orange
    [128, 0, 255], // Purple
];

/// Configuration for an individual channel in a multi-channel bioimaging overlay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelColorConfig {
    /// Channel index in the tensor's channel dimension.
    pub index: usize,
    /// Channel or marker label (e.g. "DAPI", "LaminB1", "CD3").
    pub name: String,
    /// Unique tint color in RGB [0..255].
    pub color_rgb: [u8; 3],
    /// Whether this channel is actively overlaid in the rendering pass.
    pub visible: bool,
    /// Optional custom display window `(start, end)` for intensity normalization.
    pub window: Option<(f32, f32)>,
    /// How the normalized intensity maps onto the tint (OMERO family, inversion).
    pub display: ChannelDisplay,
}

/// How a channel's normalized intensity maps onto its tint, after its window
/// (OMERO `family`, `coefficient` and `inverted`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelDisplay {
    /// The values the window may span (OMERO `min`..`max`), for its slider.
    pub range: Option<(f32, f32)>,
    pub inverted: bool,
    pub family: ChannelFamily,
    /// The family's curve parameter (exponent, growth), positive.
    pub coefficient: f32,
}

impl Default for ChannelDisplay {
    fn default() -> Self {
        Self {
            range: None,
            inverted: false,
            family: ChannelFamily::Linear,
            coefficient: 1.0,
        }
    }
}

impl ChannelDisplay {
    /// Maps a windowed intensity `norm` (0..1) through the family curve, then
    /// inverts it. Every curve keeps 0 at 0 and 1 at 1.
    pub fn map(&self, norm: f32) -> f32 {
        if self.family == ChannelFamily::Linear && !self.inverted {
            return norm.clamp(0.0, 1.0);
        }
        let n = norm.clamp(0.0, 1.0);
        let c = if self.coefficient.is_finite() && self.coefficient > 0.0 {
            self.coefficient
        } else {
            1.0
        };
        let curved = match self.family {
            ChannelFamily::Linear => n,
            ChannelFamily::Polynomial => n.powf(c),
            ChannelFamily::Exponential => (c * n).exp_m1() / c.exp_m1(),
            ChannelFamily::Logarithmic => (c.exp_m1() * n).ln_1p() / c,
        };
        let curved = curved.clamp(0.0, 1.0);
        if self.inverted { 1.0 - curved } else { curved }
    }
}

impl ChannelColorConfig {
    /// Create a new channel configuration with default visibility and color.
    pub fn new(index: usize, name: String, color_rgb: [u8; 3]) -> Self {
        Self {
            index,
            name,
            color_rgb,
            visible: true,
            window: None,
            display: ChannelDisplay::default(),
        }
    }
}

/// Parse a hex color string (e.g. `"0000FF"`, `"#FFFF00"`, `"FF00FF"`, `"FF0000FF"`) into `[u8; 3]`.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 || clean.len() == 8 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        return Some([r, g, b]);
    }
    None
}
