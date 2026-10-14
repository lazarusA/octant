//! Multi-channel bioimaging 3D volumetric additive overlay slicing with per-channel color tints.

use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::blend_channels;

/// Slices an N-dimensional `OctantBlock` into a 3D `VolumeData` with unique per-channel color tints additively.
#[allow(clippy::too_many_arguments)]
pub fn slice_multichannel_volume_composite_nd(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
    fixed_indices: &[usize],
    channel_configs: &[ChannelColorConfig],
    dataset_name: &str,
) -> Option<VolumeData> {
    if block.shape.len() < 3 || c_dim >= block.shape.len() {
        return None;
    }
    let nx = x_range.1.saturating_sub(x_range.0).max(1);
    let ny = y_range.1.saturating_sub(y_range.0).max(1);
    let nz = z_range.1.saturating_sub(z_range.0).max(1);
    let len = nx.checked_mul(ny)?.checked_mul(nz)?;
    let has_z = z_dim < block.shape.len() && z_dim != c_dim;
    let z_dim = if has_z { z_dim } else { usize::MAX };
    let values = blend_channels(
        block,
        c_dim,
        fixed_indices,
        channel_configs,
        len,
        |fixed, cfg| {
            let (x, y, z) = (x_range, y_range, z_range);
            let data =
                block.volume_with_ranges(x_dim, y_dim, z_dim, x, y, z, fixed, &cfg.name, true)?;
            Some(data.values)
        },
    )?;
    Some(VolumeData::new(
        nx,
        ny,
        nz,
        values,
        0.0,
        16777215.0,
        dataset_name.to_string(),
    ))
}
