//! Multi-channel bioimaging 3D volumetric additive overlay slicing with per-channel color tints.

use crate::data::octant_block::OctantBlock;
use crate::data::volume_data::VolumeData;

use super::types::ChannelColorConfig;
use super::utils::{compute_channel_normalization, normalize_channel_value, pack_rgb};

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

    for (cfg, local_c) in visible_configs {
        let mut fixed = fixed_indices.to_vec();
        if fixed.len() < block.shape.len() {
            fixed.resize(block.shape.len(), 0);
        }
        fixed[c_dim] = local_c;

        if let Some(vdata) = block.volume_with_ranges(
            x_dim, y_dim, eff_z_dim, x_range, y_range, z_range, &fixed, &cfg.name, true,
        ) {
            channel_loaded = true;
            accumulate_channel_voxels(
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

    let values = finalize_composite_voxels(total_voxels, &acc_r, &acc_g, &acc_b, &any_valid);

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

fn accumulate_channel_voxels(
    voxels: &[f32],
    cfg: &ChannelColorConfig,
    acc_r: &mut [f32],
    acc_g: &mut [f32],
    acc_b: &mut [f32],
    any_valid: &mut [bool],
) {
    let (scale, offset, is_i8) = compute_channel_normalization(voxels, cfg.window, 1.0);
    let col = [
        cfg.color_rgb[0] as f32,
        cfg.color_rgb[1] as f32,
        cfg.color_rgb[2] as f32,
    ];

    for (i, &raw) in voxels.iter().enumerate() {
        if !raw.is_nan() && i < acc_r.len() {
            let norm = normalize_channel_value(raw, scale, offset, is_i8, 1.0);
            acc_r[i] += norm * col[0];
            acc_g[i] += norm * col[1];
            acc_b[i] += norm * col[2];
            any_valid[i] = true;
        }
    }
}

fn finalize_composite_voxels(
    total_voxels: usize,
    acc_r: &[f32],
    acc_g: &[f32],
    acc_b: &[f32],
    any_valid: &[bool],
) -> Vec<f32> {
    let mut values = Vec::with_capacity(total_voxels);
    for i in 0..total_voxels {
        if !any_valid.get(i).copied().unwrap_or(false) {
            values.push(f32::NAN);
        } else {
            let r = acc_r.get(i).copied().unwrap_or(0.0).clamp(0.0, 255.0);
            let g = acc_g.get(i).copied().unwrap_or(0.0).clamp(0.0, 255.0);
            let b = acc_b.get(i).copied().unwrap_or(0.0).clamp(0.0, 255.0);
            values.push(pack_rgb(r, g, b));
        }
    }
    values
}
