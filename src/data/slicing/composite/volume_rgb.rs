//! TrueColor RGB composite slicing for 3D volumetric datasets.

use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;

use super::cmyk::{blend_cmyk, is_cmyk_block};
use super::probe::local_channel;
use super::utils::compute_channel_normalization;

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
    let channel_vol = |ch: Option<usize>| {
        extract_channel_vol(
            block,
            c_dim,
            x_dim,
            y_dim,
            eff_z_dim,
            x_range,
            y_range,
            z_range,
            fixed_indices,
            ch,
        )
    };

    // CMYK inks are bands 0-3 (C, M, Y, K), which `is_cmyk_block` checks the block holds.
    let values = if is_cmyk_block(block, c_dim) {
        let ink = |k: usize| channel_vol(Some(k));
        let (c, m, y, k) = (ink(0)?, ink(1)?, ink(2)?, ink(3)?);
        blend_cmyk([&c, &m, &y, &k], total_voxels)?
    } else {
        let (r_v, r_scale, r_off, r_i8) = norm_volume_info(channel_vol(channels[0]));
        let (g_v, g_scale, g_off, g_i8) = norm_volume_info(channel_vol(channels[1]));
        let (b_v, b_scale, b_off, b_i8) = norm_volume_info(channel_vol(channels[2]));
        super::rgb::blend_rgb_pixels(
            total_voxels,
            (r_v.as_deref(), r_scale, r_off, r_i8),
            (g_v.as_deref(), g_scale, g_off, g_i8),
            (b_v.as_deref(), b_scale, b_off, b_i8),
        )
    };

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
fn extract_channel_vol(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    eff_z_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
    fixed_indices: &[usize],
    ch_opt: Option<usize>,
) -> Option<Vec<f32>> {
    let ch = ch_opt?;
    let c_start = block.origin.get(c_dim).copied().unwrap_or(0);
    let num_channels = block.shape.get(c_dim).copied().unwrap_or(1);
    let local_ch = local_channel(ch, c_start, num_channels);
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
}

fn norm_volume_info(vol_opt: Option<Vec<f32>>) -> (Option<Vec<f32>>, f32, f32, bool) {
    if let Some(ref v) = vol_opt {
        let (scale, offset, is_i8) = compute_channel_normalization(v, None, 255.0);
        (vol_opt, scale, offset, is_i8)
    } else {
        (None, 1.0, 0.0, false)
    }
}
