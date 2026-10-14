//! Utility functions for color space conversion, normalization, and bit packing.

use super::types::ChannelColorConfig;

/// Normalization scale and offset computation for diverse numeric ranges.
pub fn compute_normalization_scale(
    g_min: f32,
    g_max: f32,
    is_i8: bool,
    target_max: f32,
) -> (f32, f32) {
    if is_i8 {
        (target_max / 255.0, 0.0)
    } else if g_min >= 0.0 && g_max <= 1.0 {
        (target_max, 0.0)
    } else if g_min >= 0.0 && g_max <= 255.0 {
        (target_max / 255.0, 0.0)
    } else if g_max > g_min {
        (target_max / (g_max - g_min), g_min)
    } else {
        (1.0, 0.0)
    }
}

/// Unified normalization parameters (scale, offset, is_i8) for a channel slice/volume.
pub fn compute_channel_normalization(
    values: &[f32],
    window: Option<(f32, f32)>,
    target_max: f32,
) -> (f32, f32, bool) {
    if let Some((w_start, w_end)) = window {
        let is_i8 = (-128.0..0.0).contains(&w_start) && w_end <= 127.0;
        let scale = if w_end > w_start {
            target_max / (w_end - w_start)
        } else {
            0.0
        };
        (scale, w_start, is_i8)
    } else {
        let (min_v, max_v) = crate::utils::compute_finite_min_max(values);
        let is_i8 = (-128.0..0.0).contains(&min_v) && max_v <= 127.0;
        let (scale, offset) = compute_normalization_scale(min_v, max_v, is_i8, target_max);
        (scale, offset, is_i8)
    }
}

/// Normalize a raw float value using precomputed scale, offset, and int8 flags.
#[inline(always)]
pub fn normalize_channel_value(
    raw: f32,
    scale: f32,
    offset: f32,
    is_i8: bool,
    max_val: f32,
) -> f32 {
    if raw.is_nan() {
        f32::NAN
    } else if is_i8 {
        let raw_u = (raw as i8 as u8) as f32;
        ((raw_u - offset) * scale).clamp(0.0, max_val)
    } else {
        ((raw - offset) * scale).clamp(0.0, max_val)
    }
}

/// Convert linear color component to sRGB color component in range [0.0, 255.0].
#[inline(always)]
pub fn linear_to_srgb(linear: f32) -> f32 {
    let l = linear.clamp(0.0, 1.0);
    let srgb = if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).clamp(0.0, 255.0)
}

/// Pack 3 color components in [0.0, 255.0] into a 24-bit TrueColor float representation.
#[inline(always)]
pub fn pack_rgb(r: f32, g: f32, b: f32) -> f32 {
    let packed = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
    packed as f32
}

/// Unpack a 24-bit TrueColor float representation into (r, g, b) in [0.0, 255.0].
#[inline(always)]
pub fn unpack_rgb(packed: f32) -> (f32, f32, f32) {
    let raw = packed as u32;
    (
        (raw & 0xFF) as f32,
        ((raw >> 8) & 0xFF) as f32,
        ((raw >> 16) & 0xFF) as f32,
    )
}

/// Filters and sorts visible channel configurations overlapping `[c_start, c_start + num_channels)`.
pub fn visible_channel_configs(
    configs: &[ChannelColorConfig],
    c_start: usize,
    num_channels: usize,
) -> Vec<(&ChannelColorConfig, usize)> {
    let mut visible: Vec<_> = configs
        .iter()
        .filter(|c| c.visible)
        .filter_map(|c| {
            let local_c = c.index.checked_sub(c_start)?;
            if local_c < num_channels {
                Some((c, local_c))
            } else {
                None
            }
        })
        .collect();
    visible.sort_by_key(|(c, _)| c.index);
    visible
}

/// Channels added up in their tints, then packed into 24-bit TrueColor.
pub struct ChannelAccumulator {
    rgb: Vec<[f32; 3]>,
    valid: Vec<bool>,
    loaded: bool,
}

impl ChannelAccumulator {
    /// An accumulator of `len` cells.
    pub fn new(len: usize) -> Self {
        Self {
            rgb: vec![[0.0; 3]; len],
            valid: vec![false; len],
            loaded: false,
        }
    }

    /// Adds one channel's values, normalized by its window and display curve
    /// and tinted by its color.
    pub fn add(&mut self, values: &[f32], cfg: &ChannelColorConfig) {
        self.loaded = true;
        let (scale, offset, is_i8) = compute_channel_normalization(values, cfg.window, 1.0);
        let curve = cfg.display.curve();
        let tint = cfg.color_rgb.map(f32::from);
        for ((raw, rgb), valid) in values.iter().zip(&mut self.rgb).zip(&mut self.valid) {
            if !raw.is_nan() {
                let norm = curve.map(normalize_channel_value(*raw, scale, offset, is_i8, 1.0));
                for (acc, t) in rgb.iter_mut().zip(tint) {
                    *acc += norm * t;
                }
                *valid = true;
            }
        }
    }

    /// The packed colors, `NAN` where no channel had a value; `None` when no
    /// channel was added.
    pub fn finish(self) -> Option<Vec<f32>> {
        if !self.loaded {
            return None;
        }
        let pack = |([r, g, b], valid): ([f32; 3], bool)| {
            if valid {
                pack_rgb(
                    r.clamp(0.0, 255.0),
                    g.clamp(0.0, 255.0),
                    b.clamp(0.0, 255.0),
                )
            } else {
                f32::NAN
            }
        };
        Some(self.rgb.into_iter().zip(self.valid).map(pack).collect())
    }
}
