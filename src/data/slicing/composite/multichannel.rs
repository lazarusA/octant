//! Multi-channel bioimaging additive overlay slicing with per-channel unique color tints.

use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::blend_channels;

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
    let width = x_range.1.saturating_sub(x_range.0).max(1);
    let height = y_range.1.saturating_sub(y_range.0).max(1);
    let values = blend_channels(
        block,
        c_dim,
        fixed_indices,
        channel_configs,
        width.checked_mul(height)?,
        |fixed, cfg| {
            let data = block
                .slice_2d_with_ranges(x_dim, y_dim, x_range, y_range, fixed, 1, &cfg.name, true)?;
            Some(data.values)
        },
    )?;
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
