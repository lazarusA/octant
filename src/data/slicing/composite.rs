//! RGB and CMYK composite slicing to 24-bit TrueColor `MatrixData`.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

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
        let x_range = (0, width.saturating_sub(1));
        let y_range = (0, height.saturating_sub(1));
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

fn slice_cmyk_composite(
    block: &OctantBlock,
    width: usize,
    height: usize,
    plane_size: usize,
    anim_extent: usize,
) -> Option<MatrixData> {
    if 4 * plane_size > block.values.len() {
        return None;
    }
    let (c, m, y, k) = (
        &block.values[0..plane_size],
        &block.values[plane_size..2 * plane_size],
        &block.values[2 * plane_size..3 * plane_size],
        &block.values[3 * plane_size..4 * plane_size],
    );
    let (c_min, c_max) = crate::utils::compute_finite_min_max(c);
    let (m_min, m_max) = crate::utils::compute_finite_min_max(m);
    let (y_min, y_max) = crate::utils::compute_finite_min_max(y);
    let (k_min, k_max) = crate::utils::compute_finite_min_max(k);

    let g_min = c_min.min(m_min).min(y_min).min(k_min);
    let g_max = c_max.max(m_max).max(y_max).max(k_max);
    let is_i8_cmyk = (-128.0..0.0).contains(&g_min) && g_max <= 127.0;

    let (scale, offset) = compute_normalization_scale(g_min, g_max, is_i8_cmyk, 1.0);
    let mut values = Vec::with_capacity(plane_size);

    for i in 0..plane_size {
        if c[i].is_nan() || m[i].is_nan() || y[i].is_nan() || k[i].is_nan() {
            values.push(f32::NAN);
        } else {
            let (raw_c, raw_m, raw_y, raw_k) = if is_i8_cmyk {
                (
                    (c[i] as i8 as u8) as f32,
                    (m[i] as i8 as u8) as f32,
                    (y[i] as i8 as u8) as f32,
                    (k[i] as i8 as u8) as f32,
                )
            } else {
                (c[i], m[i], y[i], k[i])
            };
            let c_n = ((raw_c - offset) * scale).clamp(0.0, 1.0);
            let m_n = ((raw_m - offset) * scale).clamp(0.0, 1.0);
            let y_n = ((raw_y - offset) * scale).clamp(0.0, 1.0);
            let k_n = ((raw_k - offset) * scale).clamp(0.0, 1.0);

            let r = linear_to_srgb((1.0 - c_n) * (1.0 - k_n));
            let g = linear_to_srgb((1.0 - m_n) * (1.0 - k_n));
            let b = linear_to_srgb((1.0 - y_n) * (1.0 - k_n));
            values.push(pack_rgb(r, g, b));
        }
    }

    Some(MatrixData::new(
        width,
        height,
        values,
        0.0,
        16777215.0,
        format!("{} (CMYK Composite)", block.variable_name),
        anim_extent,
    ))
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

    let width = (x_range.1.saturating_sub(x_range.0) + 1).max(1);
    let height = (y_range.1.saturating_sub(y_range.0) + 1).max(1);
    let plane_size = width.checked_mul(height)?;

    let extract_plane = |ch_opt: Option<usize>| -> Option<Vec<f32>> {
        let ch = ch_opt?.min(num_channels.saturating_sub(1));
        let mut fixed = fixed_indices.to_vec();
        if fixed.len() < block.shape.len() {
            fixed.resize(block.shape.len(), 0);
        }
        fixed[c_dim] = ch;
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
    };

    let r_plane = extract_plane(channels[0]);
    let g_plane = extract_plane(channels[1]);
    let b_plane = extract_plane(channels[2]);

    let norm_plane = |plane_opt: Option<Vec<f32>>| -> (Option<Vec<f32>>, f32, f32) {
        if let Some(ref p) = plane_opt {
            let (min_v, max_v) = crate::utils::compute_finite_min_max(p);
            let is_i8 = (-128.0..0.0).contains(&min_v) && max_v <= 127.0;
            let (scale, offset) = compute_normalization_scale(min_v, max_v, is_i8, 255.0);
            (plane_opt, scale, offset)
        } else {
            (None, 1.0, 0.0)
        }
    };

    let (r_p, r_scale, r_off) = norm_plane(r_plane);
    let (g_p, g_scale, g_off) = norm_plane(g_plane);
    let (b_p, b_scale, b_off) = norm_plane(b_plane);

    let mut values = Vec::with_capacity(plane_size);
    for i in 0..plane_size {
        let r_val = r_p.as_ref().and_then(|p| p.get(i).copied()).unwrap_or(0.0);
        let g_val = g_p.as_ref().and_then(|p| p.get(i).copied()).unwrap_or(0.0);
        let b_val = b_p.as_ref().and_then(|p| p.get(i).copied()).unwrap_or(0.0);

        if r_val.is_nan() && g_val.is_nan() && b_val.is_nan() {
            values.push(f32::NAN);
        } else {
            let r_n = if r_p.is_some() && !r_val.is_nan() {
                ((r_val - r_off) * r_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            let g_n = if g_p.is_some() && !g_val.is_nan() {
                ((g_val - g_off) * g_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            let b_n = if b_p.is_some() && !b_val.is_nan() {
                ((b_val - b_off) * b_scale).clamp(0.0, 255.0)
            } else {
                0.0
            };
            values.push(pack_rgb(r_n, g_n, b_n));
        }
    }

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

fn compute_normalization_scale(g_min: f32, g_max: f32, is_i8: bool, target_max: f32) -> (f32, f32) {
    if is_i8 {
        (target_max / 255.0, 0.0)
    } else if g_min >= 0.0 && g_max <= 1.0 {
        (target_max, 0.0)
    } else if g_min >= 0.0 && g_max <= 255.0 {
        (target_max / 255.0, 0.0)
    } else if g_min >= 0.0 && g_max <= 65535.0 {
        (target_max / 65535.0, 0.0)
    } else if g_max > g_min {
        (target_max / (g_max - g_min), g_min)
    } else {
        (1.0, 0.0)
    }
}

#[inline(always)]
fn linear_to_srgb(linear: f32) -> f32 {
    let l = linear.clamp(0.0, 1.0);
    let srgb = if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).clamp(0.0, 255.0)
}

#[inline(always)]
fn pack_rgb(r: f32, g: f32, b: f32) -> f32 {
    let packed = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
    packed as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_pack_unpack_fidelity() {
        let test_colors = [
            (255.0, 255.0, 255.0),
            (255.0, 254.0, 255.0),
            (254.0, 255.0, 255.0),
            (255.0, 255.0, 254.0),
            (0.0, 0.0, 0.0),
            (255.0, 0.0, 0.0),
            (0.0, 255.0, 0.0),
            (0.0, 0.0, 255.0),
            (128.0, 128.0, 128.0),
        ];

        for (r, g, b) in test_colors {
            let packed_f32 = pack_rgb(r, g, b);
            let raw_u32 = packed_f32 as u32;
            let unpacked_r = (raw_u32 & 0xFF) as f32;
            let unpacked_g = ((raw_u32 >> 8) & 0xFF) as f32;
            let unpacked_b = ((raw_u32 >> 16) & 0xFF) as f32;

            assert_eq!(
                (unpacked_r, unpacked_g, unpacked_b),
                (r, g, b),
                "Fidelity mismatch for color ({r}, {g}, {b})"
            );
        }
    }
}
