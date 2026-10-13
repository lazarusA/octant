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

/// Accumulates a single channel's normalized and tinted values into RGB and validity buffers.
pub fn accumulate_channel_values(
    values: &[f32],
    cfg: &ChannelColorConfig,
    acc_r: &mut [f32],
    acc_g: &mut [f32],
    acc_b: &mut [f32],
    any_valid: &mut [bool],
) {
    let (scale, offset, is_i8) = compute_channel_normalization(values, cfg.window, 1.0);
    let col = [
        cfg.color_rgb[0] as f32,
        cfg.color_rgb[1] as f32,
        cfg.color_rgb[2] as f32,
    ];

    let len = values
        .len()
        .min(acc_r.len())
        .min(acc_g.len())
        .min(acc_b.len())
        .min(any_valid.len());

    for ((((raw, r), g), b), valid) in values[..len]
        .iter()
        .zip(&mut acc_r[..len])
        .zip(&mut acc_g[..len])
        .zip(&mut acc_b[..len])
        .zip(&mut any_valid[..len])
    {
        if !raw.is_nan() {
            let norm = cfg
                .display
                .map(normalize_channel_value(*raw, scale, offset, is_i8, 1.0));
            *r += norm * col[0];
            *g += norm * col[1];
            *b += norm * col[2];
            *valid = true;
        }
    }
}

/// Finalizes accumulated RGB components into 24-bit TrueColor packed floats or `NAN`.
pub fn finalize_composite_packed_rgb(
    total: usize,
    acc_r: &[f32],
    acc_g: &[f32],
    acc_b: &[f32],
    any_valid: &[bool],
) -> Vec<f32> {
    let len = total
        .min(acc_r.len())
        .min(acc_g.len())
        .min(acc_b.len())
        .min(any_valid.len());
    let mut values = Vec::with_capacity(total);

    for (((&valid, &r), &g), &b) in any_valid[..len]
        .iter()
        .zip(&acc_r[..len])
        .zip(&acc_g[..len])
        .zip(&acc_b[..len])
    {
        if !valid {
            values.push(f32::NAN);
        } else {
            values.push(pack_rgb(
                r.clamp(0.0, 255.0),
                g.clamp(0.0, 255.0),
                b.clamp(0.0, 255.0),
            ));
        }
    }
    values
}
