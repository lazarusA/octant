//! Types and configuration structures for multi-channel color compositing.

use serde::{Deserialize, Serialize};

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
        }
    }
}

/// Parse a hex color string (e.g. `"0000FF"`, `"#FFFF00"`, `"FF00FF"`) into `[u8; 3]`.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some([r, g, b])
}
