//! Multi-channel bioimaging 3D volumetric additive overlay slicing with per-channel color tints.

use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;

use super::types::ChannelColorConfig;
use super::utils::{
    accumulate_channel_values, finalize_composite_packed_rgb, visible_channel_configs,
};

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
    let c_start = block.origin.get(c_dim).copied().unwrap_or(0);
    let num_channels = block.shape[c_dim];
    let visible_configs = visible_channel_configs(channel_configs, c_start, num_channels);

    let nx = x_range.1.saturating_sub(x_range.0).max(1);
    let ny = y_range.1.saturating_sub(y_range.0).max(1);
    let nz = z_range.1.saturating_sub(z_range.0).max(1);
    let total_voxels = nx.checked_mul(ny)?.checked_mul(nz)?;

    if visible_configs.is_empty() {
        return Some(VolumeData::new(
            nx,
            ny,
            nz,
            vec![f32::NAN; total_voxels],
            0.0,
            16777215.0,
            dataset_name.to_string(),
        ));
    }

    let has_z = z_dim < block.shape.len() && z_dim != c_dim;
    let eff_z_dim = if has_z { z_dim } else { usize::MAX };

    let mut acc_r = vec![0.0f32; total_voxels];
    let mut acc_g = vec![0.0f32; total_voxels];
    let mut acc_b = vec![0.0f32; total_voxels];
    let mut any_valid = vec![false; total_voxels];
    let mut channel_loaded = false;

    let mut fixed = fixed_indices.to_vec();
    if fixed.len() < block.shape.len() {
        fixed.resize(block.shape.len(), 0);
    }

    for (cfg, local_c) in visible_configs {
        fixed[c_dim] = local_c;

        if let Some(vdata) = block.volume_with_ranges(
            x_dim, y_dim, eff_z_dim, x_range, y_range, z_range, &fixed, &cfg.name, true,
        ) {
            channel_loaded = true;
            accumulate_channel_values(
                &vdata.values,
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

    let values = finalize_composite_packed_rgb(total_voxels, &acc_r, &acc_g, &acc_b, &any_valid);

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
