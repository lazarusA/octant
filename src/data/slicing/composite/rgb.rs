//! Standard 3-slot RGB composite slicing for multi-band imagery and tensors.

use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::cmyk::{blend_cmyk_planes, is_cmyk_block, slice_cmyk_composite};
use super::probe::local_channel;
use super::stretch::CompositeStretch;
use super::utils::{normalize_channel_value, pack_rgb};

/// Slice an `OctantBlock` into a 24-bit TrueColor composite `MatrixData`.
pub fn slice_rgb_composite(
    block: &OctantBlock,
    channels: [usize; 3],
    anim_extent: usize,
) -> Option<MatrixData> {
    if block.shape.len() < 3 {
        return None;
    }
    let (height, width) = (
        block.shape[block.shape.len() - 2],
        block.shape[block.shape.len() - 1],
    );
    let plane_size = height.checked_mul(width)?;

    if is_cmyk_block(block, 0) {
        slice_cmyk_composite(block, width, height, plane_size, anim_extent)
    } else {
        let mut stretch = CompositeStretch::default();
        let opt_channels = [Some(channels[0]), Some(channels[1]), Some(channels[2])];
        let x_dim = block.shape.len() - 1;
        let y_dim = block.shape.len() - 2;
        let x_range = (0, width);
        let y_range = (0, height);
        let fixed = vec![0; block.shape.len()];
        slice_rgb_composite_nd(
            block,
            0,
            x_dim,
            y_dim,
            x_range,
            y_range,
            &fixed,
            opt_channels,
            anim_extent,
            &mut stretch,
        )
    }
}

/// Slices an N-dimensional `OctantBlock` with a channel dimension into TrueColor RGB `MatrixData`,
/// each channel (or CMYK's inks together) stretched as `stretch` records, the others over
/// their values and recorded there.
#[allow(clippy::too_many_arguments)]
pub fn slice_rgb_composite_nd(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    fixed_indices: &[usize],
    channels: [Option<usize>; 3],
    anim_extent: usize,
    stretch: &mut CompositeStretch,
) -> Option<MatrixData> {
    let num_channels = *block.shape.get(c_dim)?;
    if block.shape.len() < 2 || (num_channels < 2 && channels.iter().all(Option::is_none)) {
        return None;
    }

    let (x_range, y_range) = (
        clamp_window(block, x_dim, x_range),
        clamp_window(block, y_dim, y_range),
    );
    let width = x_range.1.saturating_sub(x_range.0).max(1);
    let height = y_range.1.saturating_sub(y_range.0).max(1);

    let plane = |ch: Option<usize>| {
        extract_channel_plane(
            block,
            c_dim,
            x_dim,
            y_dim,
            x_range,
            y_range,
            fixed_indices,
            ch,
        )
    };

    // CMYK inks are bands 0-3 (C, M, Y, K), which `is_cmyk_block` checks the block holds.
    if is_cmyk_block(block, c_dim) {
        let ink = |k: usize| plane(Some(k));
        let (c, m, y, k) = (ink(0)?, ink(1)?, ink(2)?, ink(3)?);
        let planes = [c.as_slice(), &m, &y, &k];
        return blend_cmyk_planes(
            planes,
            width,
            height,
            &block.variable_name,
            anim_extent,
            stretch,
        );
    }

    let name = format!("{} (RGB Composite)", block.variable_name);
    let planes = channels.map(|ch| Some((ch?, plane(ch)?)));
    rgb_matrix(planes, (width, height), name, anim_extent, stretch)
}

/// `range` along `dim` clamped into the block, keeping at least one index.
fn clamp_window(block: &OctantBlock, dim: usize, (start, end): (usize, usize)) -> (usize, usize) {
    let len = block.shape.get(dim).copied().unwrap_or(1).max(1);
    let end = end.min(len);
    (start.min(end.saturating_sub(1)), end)
}

/// The packed RGB matrix of three channels' planes, stretched as `stretch` says (`None`
/// channels stay dark).
fn rgb_matrix(
    [r_plane, g_plane, b_plane]: [Option<(usize, Vec<f32>)>; 3],
    (width, height): (usize, usize),
    name: String,
    anim_extent: usize,
    stretch: &mut CompositeStretch,
) -> Option<MatrixData> {
    let (r_p, r_scale, r_off, r_i8) = norm_plane_info(r_plane, stretch);
    let (g_p, g_scale, g_off, g_i8) = norm_plane_info(g_plane, stretch);
    let (b_p, b_scale, b_off, b_i8) = norm_plane_info(b_plane, stretch);
    let values = blend_rgb_pixels(
        width.checked_mul(height)?,
        (r_p.as_deref(), r_scale, r_off, r_i8),
        (g_p.as_deref(), g_scale, g_off, g_i8),
        (b_p.as_deref(), b_scale, b_off, b_i8),
    );
    Some(MatrixData::new(
        width,
        height,
        values,
        0.0,
        16777215.0,
        name,
        anim_extent,
    ))
}

#[allow(clippy::too_many_arguments)]
fn extract_channel_plane(
    block: &OctantBlock,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
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
    let mdata = block.slice_2d_with_ranges(
        x_dim,
        y_dim,
        x_range,
        y_range,
        &fixed,
        1,
        "composite_ch",
        true,
    )?;
    Some(mdata.values)
}

fn norm_plane_info(
    plane: Option<(usize, Vec<f32>)>,
    stretch: &mut CompositeStretch,
) -> (Option<Vec<f32>>, f32, f32, bool) {
    match plane {
        Some((channel, p)) => {
            let s = stretch.channel(channel, &p, None, 255.0);
            (Some(p), s.scale, s.offset, s.is_i8)
        }
        None => (None, 1.0, 0.0, false),
    }
}

type ChannelPlaneTuple<'a> = (Option<&'a [f32]>, f32, f32, bool);

fn blend_rgb_pixels(
    plane_size: usize,
    r: ChannelPlaneTuple,
    g: ChannelPlaneTuple,
    b: ChannelPlaneTuple,
) -> Vec<f32> {
    let mut values = Vec::with_capacity(plane_size);
    for i in 0..plane_size {
        let r_val = r.0.and_then(|p| p.get(i).copied()).unwrap_or(f32::NAN);
        let g_val = g.0.and_then(|p| p.get(i).copied()).unwrap_or(f32::NAN);
        let b_val = b.0.and_then(|p| p.get(i).copied()).unwrap_or(f32::NAN);

        if r_val.is_nan() && g_val.is_nan() && b_val.is_nan() {
            values.push(f32::NAN);
        } else {
            let r_n = if r.0.is_some() && !r_val.is_nan() {
                normalize_channel_value(r_val, r.1, r.2, r.3, 255.0)
            } else {
                0.0
            };
            let g_n = if g.0.is_some() && !g_val.is_nan() {
                normalize_channel_value(g_val, g.1, g.2, g.3, 255.0)
            } else {
                0.0
            };
            let b_n = if b.0.is_some() && !b_val.is_nan() {
                normalize_channel_value(b_val, b.1, b.2, b.3, 255.0)
            } else {
                0.0
            };
            values.push(pack_rgb(r_n, g_n, b_n));
        }
    }
    values
}
