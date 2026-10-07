//! Standard 3-slot RGB composite slicing for multi-band imagery and tensors.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::cmyk::slice_cmyk_composite;
use super::probe::local_channel;
use super::utils::{compute_channel_normalization, normalize_channel_value, pack_rgb};

/// Slice an `OctantBlock` into a 24-bit TrueColor composite `MatrixData`.
pub fn slice_rgb_composite(
    block: &OctantBlock,
    channels: [usize; 3],
    anim_extent: usize,
) -> Option<MatrixData> {
    if block.shape.len() < 3 {
        return None;
    }
    let (num_bands, height, width) = (
        block.shape[0],
        block.shape[block.shape.len() - 2],
        block.shape[block.shape.len() - 1],
    );
    let plane_size = height.checked_mul(width)?;

    let is_cmyk = num_bands >= 4
        && block
            .attributes
            .get("photometric")
            .or_else(|| block.attributes.get("color_space"))
            .is_some_and(|s| s.eq_ignore_ascii_case("cmyk"));

    if is_cmyk {
        slice_cmyk_composite(block, width, height, plane_size, anim_extent)
    } else {
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
        )
    }
}

/// Slices an N-dimensional `OctantBlock` with a channel dimension into TrueColor RGB `MatrixData`.
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
) -> Option<MatrixData> {
    if block.shape.len() < 2 || c_dim >= block.shape.len() {
        return None;
    }
    let num_channels = block.shape[c_dim];
    if num_channels < 2 && channels[0].is_none() && channels[1].is_none() && channels[2].is_none() {
        return None;
    }

    let width = x_range.1.saturating_sub(x_range.0).max(1);
    let height = y_range.1.saturating_sub(y_range.0).max(1);
    let plane_size = width.checked_mul(height)?;

    let r_plane = extract_channel_plane(
        block,
        c_dim,
        x_dim,
        y_dim,
        x_range,
        y_range,
        fixed_indices,
        channels[0],
    );
    let g_plane = extract_channel_plane(
        block,
        c_dim,
        x_dim,
        y_dim,
        x_range,
        y_range,
        fixed_indices,
        channels[1],
    );
    let b_plane = extract_channel_plane(
        block,
        c_dim,
        x_dim,
        y_dim,
        x_range,
        y_range,
        fixed_indices,
        channels[2],
    );

    let (r_p, r_scale, r_off, r_i8) = norm_plane_info(r_plane);
    let (g_p, g_scale, g_off, g_i8) = norm_plane_info(g_plane);
    let (b_p, b_scale, b_off, b_i8) = norm_plane_info(b_plane);

    let values = blend_rgb_pixels(
        plane_size,
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
        format!("{} (RGB Composite)", block.variable_name),
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

fn norm_plane_info(plane_opt: Option<Vec<f32>>) -> (Option<Vec<f32>>, f32, f32, bool) {
    if let Some(ref p) = plane_opt {
        let (scale, offset, is_i8) = compute_channel_normalization(p, None, 255.0);
        (plane_opt, scale, offset, is_i8)
    } else {
        (None, 1.0, 0.0, false)
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
