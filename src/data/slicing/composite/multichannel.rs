//! Multi-channel bioimaging additive overlay slicing with per-channel unique color tints.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::{compute_channel_normalization, normalize_channel_value, pack_rgb};

/// Slices an N-dimensional `OctantBlock` with unique per-channel color tints additively.
#[allow(clippy::too_many_arguments)]
pub fn slice_multichannel_composite_nd(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    fixed_indices: &[usize],
    channel_configs: &[ChannelColorConfig],
    anim_extent: usize,
) -> Option<MatrixData> {
    if block.shape.len() < 2 || c_dim >= block.shape.len() {
        return None;
    }
    let c_start = block.origin.get(c_dim).copied().unwrap_or(0);
    let num_channels = block.shape[c_dim];
    let mut visible_configs: Vec<(&ChannelColorConfig, usize)> = channel_configs
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
    visible_configs.sort_by_key(|(c, _)| c.index);

    let width = x_range.1.saturating_sub(x_range.0).max(1);
    let height = y_range.1.saturating_sub(y_range.0).max(1);
    let plane_size = width.checked_mul(height)?;

    if visible_configs.is_empty() {
        return Some(MatrixData::new(
            width,
            height,
            vec![f32::NAN; plane_size],
            0.0,
            16777215.0,
            format!("{} (Multi-Channel Overlay)", block.variable_name),
            anim_extent,
        ));
    }

    let mut acc_r = vec![0.0f32; plane_size];
    let mut acc_g = vec![0.0f32; plane_size];
    let mut acc_b = vec![0.0f32; plane_size];
    let mut any_valid = vec![false; plane_size];
    let mut channel_loaded = false;

    for (cfg, local_c) in visible_configs {
        let mut fixed = fixed_indices.to_vec();
        if fixed.len() < block.shape.len() {
            fixed.resize(block.shape.len(), 0);
        }
        fixed[c_dim] = local_c;

        if let Some(mdata) =
            block.slice_2d_with_ranges(x_dim, y_dim, x_range, y_range, &fixed, 1, &cfg.name, true)
        {
            channel_loaded = true;
            accumulate_channel_pixels(
                &mdata.values,
                cfg,
                &mut acc_r,
                &mut acc_g,
                &mut acc_b,
                &mut any_valid,
            );
        }
    }

    if !channel_loaded {
        return None;
    }

    let values = finalize_composite_pixels(plane_size, &acc_r, &acc_g, &acc_b, &any_valid);

    Some(MatrixData::new(
        width,
        height,
        values,
        0.0,
        16777215.0,
        format!("{} (Multi-Channel Overlay)", block.variable_name),
        anim_extent,
    ))
}

fn accumulate_channel_pixels(
    pixels: &[f32],
    cfg: &ChannelColorConfig,
    acc_r: &mut [f32],
    acc_g: &mut [f32],
    acc_b: &mut [f32],
    any_valid: &mut [bool],
) {
    let (scale, offset, is_i8) = compute_channel_normalization(pixels, cfg.window, 1.0);
    let col = [
        cfg.color_rgb[0] as f32,
        cfg.color_rgb[1] as f32,
        cfg.color_rgb[2] as f32,
    ];

    let len = pixels
        .len()
        .min(acc_r.len())
        .min(acc_g.len())
        .min(acc_b.len())
        .min(any_valid.len());
    let p_slice = &pixels[..len];
    let r_slice = &mut acc_r[..len];
    let g_slice = &mut acc_g[..len];
    let b_slice = &mut acc_b[..len];
    let val_slice = &mut any_valid[..len];

    for ((((raw, r), g), b), valid) in p_slice
        .iter()
        .zip(r_slice.iter_mut())
        .zip(g_slice.iter_mut())
        .zip(b_slice.iter_mut())
        .zip(val_slice.iter_mut())
    {
        if !raw.is_nan() {
            let norm = normalize_channel_value(*raw, scale, offset, is_i8, 1.0);
            *r += norm * col[0];
            *g += norm * col[1];
            *b += norm * col[2];
            *valid = true;
        }
    }
}

fn finalize_composite_pixels(
    plane_size: usize,
    acc_r: &[f32],
    acc_g: &[f32],
    acc_b: &[f32],
    any_valid: &[bool],
) -> Vec<f32> {
    let len = plane_size
        .min(acc_r.len())
        .min(acc_g.len())
        .min(acc_b.len())
        .min(any_valid.len());
    let mut values = Vec::with_capacity(plane_size);

    for (((&valid, &r), &g), &b) in any_valid[..len]
        .iter()
        .zip(&acc_r[..len])
        .zip(&acc_g[..len])
        .zip(&acc_b[..len])
    {
        if !valid {
            values.push(f32::NAN);
        } else {
            let r_clamped = r.clamp(0.0, 255.0);
            let g_clamped = g.clamp(0.0, 255.0);
            let b_clamped = b.clamp(0.0, 255.0);
            values.push(pack_rgb(r_clamped, g_clamped, b_clamped));
        }
    }
    values
}
