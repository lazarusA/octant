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
    let stretch = &mut super::stretch::CompositeStretch::default();
    slice_multichannel_volume_stretched(
        block,
        (c_dim, x_dim, y_dim, z_dim),
        (x_range, y_range, z_range),
        fixed_indices,
        channel_configs,
        stretch,
    )
    .map(|v| VolumeData {
        dataset_name: dataset_name.to_string(),
        ..v
    })
}

/// [`slice_multichannel_volume_composite_nd`] with channels stretched by
/// `stretch` (recorded from the values when it holds none for a channel),
/// so bricks of one plot share their plot's colors (`app::volume_lod`).
pub fn slice_multichannel_volume_stretched(
    block: &OctantBlock,
    (c_dim, x_dim, y_dim, z_dim): (usize, usize, usize, usize),
    (x_range, y_range, z_range): ((usize, usize), (usize, usize), (usize, usize)),
    fixed_indices: &[usize],
    channel_configs: &[ChannelColorConfig],
    stretch: &mut super::stretch::CompositeStretch,
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
        stretch,
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
        String::new(),
    ))
}
