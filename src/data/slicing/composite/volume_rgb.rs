//! TrueColor RGB composite slicing for 3D volumetric datasets.

use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;

use super::cmyk::{blend_cmyk_stretched, is_cmyk_block};
use super::probe::local_channel;
use super::stretch::CompositeStretch;

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
    let ranges = (x_range, y_range, z_range);
    let stretch = &mut CompositeStretch::default();
    slice_rgb_volume_stretched(
        block,
        (c_dim, x_dim, y_dim, z_dim),
        ranges,
        fixed_indices,
        channels,
        stretch,
    )
    .map(|v| VolumeData {
        dataset_name: dataset_name.to_string(),
        ..v
    })
}

/// [`slice_rgb_volume_composite_nd`] with each channel normalized by
/// `stretch` (recorded from the values when it holds none for the channel),
/// so bricks of one plot share their plot's colors (`app::volume_lod`).
pub fn slice_rgb_volume_stretched(
    block: &OctantBlock,
    (c_dim, x_dim, y_dim, z_dim): (usize, usize, usize, usize),
    (x_range, y_range, z_range): ((usize, usize), (usize, usize), (usize, usize)),
    fixed_indices: &[usize],
    channels: [Option<usize>; 3],
    stretch: &mut CompositeStretch,
) -> Option<VolumeData> {
    let (eff_z_dim, [nx, ny, nz], total_voxels) =
        volume_extent(block, (c_dim, z_dim), (x_range, y_range, z_range), channels)?;
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
        blend_cmyk_stretched([&c, &m, &y, &k], total_voxels, stretch)?
    } else {
        let [r, g, b] = channels.map(|ch| norm_volume_info(ch, channel_vol(ch), stretch));
        super::rgb::blend_rgb_pixels(
            total_voxels,
            (r.0.as_deref(), r.1, r.2, r.3),
            (g.0.as_deref(), g.1, g.2, g.3),
            (b.0.as_deref(), b.1, b.2, b.3),
        )
    };
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

/// The z dimension a composite volume reads (`usize::MAX` when the block
/// has none), its voxels along x, y, z and their count; `None` when the
/// block holds no composite.
fn volume_extent(
    block: &OctantBlock,
    (c_dim, z_dim): (usize, usize),
    (x_range, y_range, z_range): ((usize, usize), (usize, usize), (usize, usize)),
    channels: [Option<usize>; 3],
) -> Option<(usize, [usize; 3], usize)> {
    if block.shape.len() < 3 || c_dim >= block.shape.len() {
        return None;
    }
    let num_channels = block.shape[c_dim];
    if num_channels < 2 && channels.iter().all(Option::is_none) {
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
    Some((eff_z_dim, [nx, ny, nz], total_voxels))
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

fn norm_volume_info(
    ch: Option<usize>,
    vol_opt: Option<Vec<f32>>,
    stretch: &mut CompositeStretch,
) -> (Option<Vec<f32>>, f32, f32, bool) {
    if let (Some(ch), Some(v)) = (ch, &vol_opt) {
        let s = stretch.channel(ch, v, None, 255.0);
        (vol_opt, s.scale, s.offset, s.is_i8)
    } else {
        (None, 1.0, 0.0, false)
    }
}
