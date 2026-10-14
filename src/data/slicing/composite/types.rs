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
    /// The curve with its coefficient checked, for mapping many intensities.
    pub fn curve(&self) -> ChannelCurve {
        let c = if self.coefficient.is_finite() && self.coefficient > 0.0 {
            f64::from(self.coefficient)
        } else {
            1.0
        };
        let top = OMERO_MAX.powf(c);
        ChannelCurve {
            family: self.family,
            inverted: self.inverted,
            c,
            top,
            exp_span: -(-top).exp_m1(),
        }
    }
}

/// The end of the range the window maps onto before the family's function,
/// as OMERO renders channels (`0..=255`).
const OMERO_MAX: f64 = 255.0;

/// A `ChannelDisplay`'s curve, its coefficient checked once per channel.
///
/// The curves of OMERO's channel rendering, without its 8-bit rounding: the
/// windowed intensity `n` maps onto `v = 255 n`, then onto
/// `(f(v) - f(0)) / (f(255) - f(0))` with `f` the family's map: `v^k`
/// (polynomial), `ln v` with `v <= 0` read as 1 and `k` ignored
/// (logarithmic), `exp(v^k)` (exponential).
///
/// References: the `omero` metadata (`window`, `family`, `coefficient`,
/// `inverted`) of the OME-NGFF specification,
/// <https://ngff.openmicroscopy.org/0.5/#omero-md>; the families as the
/// OMERO API documents them (`QuantumMap` and its `PolynomialMap`,
/// `LogarithmicMap`, `ExponentialMap`),
/// <https://docs.openmicroscopy.org/omero-renderer/5.6.1/javadoc/omeis/providers/re/quantum/QuantumMap.html>.
/// No publication states the formulas in full; they are written out above.
#[derive(Debug, Clone, Copy)]
pub struct ChannelCurve {
    family: ChannelFamily,
    inverted: bool,
    c: f64,
    /// `255^k`.
    top: f64,
    /// `1 - exp(-255^k)`.
    exp_span: f64,
}

impl ChannelCurve {
    /// Maps a windowed intensity `norm` (0..1) through the family curve, then
    /// inverts it. Every curve keeps 0 at 0 and 1 at 1.
    #[inline]
    pub fn map(&self, norm: f32) -> f32 {
        let n = f64::from(norm.clamp(0.0, 1.0));
        let curved = match self.family {
            ChannelFamily::Linear => n,
            ChannelFamily::Polynomial => n.powf(self.c),
            ChannelFamily::Logarithmic => (OMERO_MAX * n).max(1.0).ln() / OMERO_MAX.ln(),
            ChannelFamily::Exponential => {
                // (exp(p) - 1) / (exp(top) - 1), kept finite for large `top`.
                let p = (OMERO_MAX * n).powf(self.c);
                (p - self.top).exp() * -(-p).exp_m1() / self.exp_span
            }
        }
        .clamp(0.0, 1.0) as f32;
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
