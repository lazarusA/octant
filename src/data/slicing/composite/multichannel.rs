//! Multi-channel bioimaging additive overlay slicing with per-channel unique color tints.

use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::{
    accumulate_channel_values, finalize_composite_packed_rgb, visible_channel_configs,
};

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
    let visible_configs = visible_channel_configs(channel_configs, c_start, num_channels);

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

    let mut fixed = fixed_indices.to_vec();
    if fixed.len() < block.shape.len() {
        fixed.resize(block.shape.len(), 0);
    }

    for (cfg, local_c) in visible_configs {
        fixed[c_dim] = local_c;

        if let Some(mdata) =
            block.slice_2d_with_ranges(x_dim, y_dim, x_range, y_range, &fixed, 1, &cfg.name, true)
        {
            channel_loaded = true;
            accumulate_channel_values(
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

    let values = finalize_composite_packed_rgb(plane_size, &acc_r, &acc_g, &acc_b, &any_valid);

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
