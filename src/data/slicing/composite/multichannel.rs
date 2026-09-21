//! Multi-channel bioimaging additive overlay slicing with per-channel unique color tints.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::{compute_normalization_scale, pack_rgb};

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
    let num_channels = block.shape[c_dim];
    let visible_configs: Vec<&ChannelColorConfig> = channel_configs
        .iter()
        .filter(|c| c.visible && c.index < num_channels)
        .collect();

    if visible_configs.is_empty() {
        return None;
    }

    let width = x_range.1.saturating_sub(x_range.0).max(1);
    let height = y_range.1.saturating_sub(y_range.0).max(1);
    let plane_size = width.checked_mul(height)?;

    let mut channel_planes = Vec::with_capacity(visible_configs.len());
    for cfg in &visible_configs {
        if let Some(plane) = extract_and_normalize_channel(
            block,
            c_dim,
            x_dim,
            y_dim,
            x_range,
            y_range,
            fixed_indices,
            cfg,
        ) {
            channel_planes.push(plane);
        }
    }

    if channel_planes.is_empty() {
        return None;
    }

    let values = blend_additive_planes(&channel_planes, plane_size);

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

#[allow(clippy::too_many_arguments)]
fn extract_and_normalize_channel(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    fixed_indices: &[usize],
    cfg: &ChannelColorConfig,
) -> Option<(Vec<f32>, [f32; 3])> {
    let mut fixed = fixed_indices.to_vec();
    if fixed.len() < block.shape.len() {
        fixed.resize(block.shape.len(), 0);
    }
    fixed[c_dim] = cfg.index;

    let mdata =
        block.slice_2d_with_ranges(x_dim, y_dim, x_range, y_range, &fixed, 1, &cfg.name, true)?;

    let (scale, offset, is_i8) = if let Some((w_start, w_end)) = cfg.window {
        let is_i8 = (-128.0..0.0).contains(&w_start) && w_end <= 127.0;
        let scale = if w_end > w_start {
            1.0 / (w_end - w_start)
        } else {
            1.0
        };
        (scale, w_start, is_i8)
    } else {
        let (min_v, max_v) = crate::utils::compute_finite_min_max(&mdata.values);
        let is_i8 = (-128.0..0.0).contains(&min_v) && max_v <= 127.0;
        let (scale, offset) = compute_normalization_scale(min_v, max_v, is_i8, 1.0);
        (scale, offset, is_i8)
    };

    let color_rgb = [
        cfg.color_rgb[0] as f32,
        cfg.color_rgb[1] as f32,
        cfg.color_rgb[2] as f32,
    ];

    let norm_plane: Vec<f32> = mdata
        .values
        .into_iter()
        .map(|raw| {
            if raw.is_nan() {
                f32::NAN
            } else if is_i8 {
                let raw_u = (raw as i8 as u8) as f32;
                ((raw_u - offset) * scale).clamp(0.0, 1.0)
            } else {
                ((raw - offset) * scale).clamp(0.0, 1.0)
            }
        })
        .collect();

    Some((norm_plane, color_rgb))
}

fn blend_additive_planes(channel_planes: &[(Vec<f32>, [f32; 3])], plane_size: usize) -> Vec<f32> {
    let mut values = Vec::with_capacity(plane_size);
    for i in 0..plane_size {
        let mut acc_r = 0.0f32;
        let mut acc_g = 0.0f32;
        let mut acc_b = 0.0f32;
        let mut any_valid = false;

        for (plane, color) in channel_planes {
            let intensity = plane.get(i).copied().unwrap_or(f32::NAN);
            if !intensity.is_nan() {
                any_valid = true;
                acc_r += intensity * color[0];
                acc_g += intensity * color[1];
                acc_b += intensity * color[2];
            }
        }

        if !any_valid {
            values.push(f32::NAN);
        } else {
            let r_clamped = acc_r.clamp(0.0, 255.0);
            let g_clamped = acc_g.clamp(0.0, 255.0);
            let b_clamped = acc_b.clamp(0.0, 255.0);
            values.push(pack_rgb(r_clamped, g_clamped, b_clamped));
        }
    }
    values
}
