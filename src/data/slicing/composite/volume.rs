//! Multi-channel bioimaging 3D volumetric additive overlay slicing with per-channel color tints.

use crate::data::octant_block::OctantBlock;
use crate::data::volume_data::VolumeData;

use super::types::ChannelColorConfig;
use super::utils::{compute_normalization_scale, pack_rgb};

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

    if visible_configs.is_empty() {
        let nx = x_range.1.saturating_sub(x_range.0).max(1);
        let ny = y_range.1.saturating_sub(y_range.0).max(1);
        let nz = z_range.1.saturating_sub(z_range.0).max(1);
        let total_voxels = nx.checked_mul(ny)?.checked_mul(nz)?;
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

    let mut channel_volumes = Vec::with_capacity(visible_configs.len());
    let mut dims: Option<(usize, usize, usize)> = None;

    for (cfg, local_c) in &visible_configs {
        if let Some((v_data, color)) = extract_and_normalize_channel_volume(
            block,
            c_dim,
            x_dim,
            y_dim,
            z_dim,
            x_range,
            y_range,
            z_range,
            fixed_indices,
            cfg,
            *local_c,
        ) {
            if dims.is_none() {
                dims = Some((v_data.width, v_data.height, v_data.depth));
            }
            channel_volumes.push((v_data.values, color));
        }
    }

    if channel_volumes.is_empty() {
        return None;
    }

    let (nx, ny, nz) = dims?;
    let total_voxels = nx.checked_mul(ny)?.checked_mul(nz)?;
    let values = blend_additive_volumes(&channel_volumes, total_voxels);

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

/// Slices an N-dimensional `OctantBlock` with a channel dimension into TrueColor RGB `VolumeData`.
#[allow(clippy::too_many_arguments)]
pub fn slice_rgb_volume_composite_nd(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
    fixed_indices: &[usize],
    channels: [Option<usize>; 3],
    dataset_name: &str,
) -> Option<VolumeData> {
    if block.shape.len() < 3 || c_dim >= block.shape.len() {
        return None;
    }
    let num_channels = block.shape[c_dim];
    if num_channels < 2 && channels[0].is_none() && channels[1].is_none() && channels[2].is_none() {
        return None;
    }

    let has_z = z_dim < block.shape.len() && z_dim != c_dim;
    let eff_z_dim = if has_z { z_dim } else { usize::MAX };
    let nx = x_range.1.saturating_sub(x_range.0).max(1);
    let ny = y_range.1.saturating_sub(y_range.0).max(1);
    let nz = if has_z {
        z_range.1.saturating_sub(z_range.0).max(1)
    } else {
        1
    };
    let total_voxels = nx.checked_mul(ny)?.checked_mul(nz)?;

    let c_start = block.origin.get(c_dim).copied().unwrap_or(0);
    let extract_volume = |ch_opt: Option<usize>| -> Option<Vec<f32>> {
        let ch = ch_opt?;
        let local_ch = if ch >= c_start && ch < c_start + num_channels {
            ch - c_start
        } else {
            ch.min(num_channels.saturating_sub(1))
        };
        let mut fixed = fixed_indices.to_vec();
        if fixed.len() < block.shape.len() {
            fixed.resize(block.shape.len(), 0);
        }
        fixed[c_dim] = local_ch;
        let vdata = block.volume_with_ranges(
            x_dim,
            y_dim,
            eff_z_dim,
            x_range,
            y_range,
            z_range,
            &fixed,
            "rgb_volume_ch",
            true,
        )?;
        Some(vdata.values)
    };

    let r_vol = extract_volume(channels[0]);
    let g_vol = extract_volume(channels[1]);
    let b_vol = extract_volume(channels[2]);

    let norm_volume = |vol_opt: Option<Vec<f32>>| -> (Option<Vec<f32>>, f32, f32) {
        if let Some(ref v) = vol_opt {
            let (min_v, max_v) = crate::utils::compute_finite_min_max(v);
            let is_i8 = (-128.0..0.0).contains(&min_v) && max_v <= 127.0;
            let (scale, offset) = compute_normalization_scale(min_v, max_v, is_i8, 255.0);
            (vol_opt, scale, offset)
        } else {
            (None, 1.0, 0.0)
        }
    };

    let (r_v, r_scale, r_off) = norm_volume(r_vol);
    let (g_v, g_scale, g_off) = norm_volume(g_vol);
    let (b_v, b_scale, b_off) = norm_volume(b_vol);

    let mut values = Vec::with_capacity(total_voxels);
    for i in 0..total_voxels {
        let r_val = r_v.as_ref().and_then(|v| v.get(i).copied()).unwrap_or(0.0);
        let g_val = g_v.as_ref().and_then(|v| v.get(i).copied()).unwrap_or(0.0);
        let b_val = b_v.as_ref().and_then(|v| v.get(i).copied()).unwrap_or(0.0);

        if r_val.is_nan() && g_val.is_nan() && b_val.is_nan() {
            values.push(f32::NAN);
        } else {
            let r_n = if r_v.is_some() && !r_val.is_nan() {
                ((r_val - r_off) * r_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            let g_n = if g_v.is_some() && !g_val.is_nan() {
                ((g_val - g_off) * g_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            let b_n = if b_v.is_some() && !b_val.is_nan() {
                ((b_val - b_off) * b_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            values.push(pack_rgb(r_n, g_n, b_n));
        }
    }

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

#[allow(clippy::too_many_arguments)]
fn extract_and_normalize_channel_volume(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
    fixed_indices: &[usize],
    cfg: &ChannelColorConfig,
    local_c: usize,
) -> Option<(VolumeData, [f32; 3])> {
    let mut fixed = fixed_indices.to_vec();
    if fixed.len() < block.shape.len() {
        fixed.resize(block.shape.len(), 0);
    }
    fixed[c_dim] = local_c;

    let has_z = z_dim < block.shape.len() && z_dim != c_dim;
    let eff_z_dim = if has_z { z_dim } else { usize::MAX };

    let vdata = block.volume_with_ranges(
        x_dim, y_dim, eff_z_dim, x_range, y_range, z_range, &fixed, &cfg.name, true,
    )?;

    let (scale, offset, is_i8) = if let Some((w_start, w_end)) = cfg.window {
        let is_i8 = (-128.0..0.0).contains(&w_start) && w_end <= 127.0;
        let scale = if w_end > w_start {
            1.0 / (w_end - w_start)
        } else {
            1.0
        };
        (scale, w_start, is_i8)
    } else {
        let (min_v, max_v) = crate::utils::compute_finite_min_max(&vdata.values);
        let is_i8 = (-128.0..0.0).contains(&min_v) && max_v <= 127.0;
        let (scale, offset) = compute_normalization_scale(min_v, max_v, is_i8, 1.0);
        (scale, offset, is_i8)
    };

    let color_rgb = [
        cfg.color_rgb[0] as f32,
        cfg.color_rgb[1] as f32,
        cfg.color_rgb[2] as f32,
    ];

    let norm_values: Vec<f32> = vdata
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

    Some((
        VolumeData::new(
            vdata.width,
            vdata.height,
            vdata.depth,
            norm_values,
            0.0,
            1.0,
            cfg.name.clone(),
        ),
        color_rgb,
    ))
}

fn blend_additive_volumes(
    channel_volumes: &[(Vec<f32>, [f32; 3])],
    total_voxels: usize,
) -> Vec<f32> {
    let mut values = Vec::with_capacity(total_voxels);
    for i in 0..total_voxels {
        let mut acc_r = 0.0f32;
        let mut acc_g = 0.0f32;
        let mut acc_b = 0.0f32;
        let mut any_valid = false;

        for (voxels, color) in channel_volumes {
            let intensity = voxels.get(i).copied().unwrap_or(f32::NAN);
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
