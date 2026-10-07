//! CMYK 4-channel composite slicing to 24-bit TrueColor `MatrixData`.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::utils::{compute_normalization_scale, linear_to_srgb, pack_rgb};

/// Whether `block` holds CMYK inks along its channel dimension `c_dim`.
pub fn is_cmyk_block(block: &OctantBlock, c_dim: usize) -> bool {
    block.shape.get(c_dim).is_some_and(|&n| n >= 4)
        && block
            .attributes
            .get("photometric")
            .or_else(|| block.attributes.get("color_space"))
            .is_some_and(|s| s.eq_ignore_ascii_case("cmyk"))
}

/// Slices a 4-channel CMYK `OctantBlock` into TrueColor RGB `MatrixData`.
pub fn slice_cmyk_composite(
    block: &OctantBlock,
    width: usize,
    height: usize,
    plane_size: usize,
    anim_extent: usize,
) -> Option<MatrixData> {
    if 4 * plane_size > block.values.len() {
        return None;
    }
    let plane = |k: usize| &block.values[k * plane_size..(k + 1) * plane_size];
    let planes = [plane(0), plane(1), plane(2), plane(3)];
    blend_cmyk_planes(planes, width, height, &block.variable_name, anim_extent)
}

/// Converts four `width * height` C, M, Y, K planes into a TrueColor RGB `MatrixData`,
/// normalizing all inks by their shared range.
pub fn blend_cmyk_planes(
    [c, m, y, k]: [&[f32]; 4],
    width: usize,
    height: usize,
    variable_name: &str,
    anim_extent: usize,
) -> Option<MatrixData> {
    let plane_size = width.checked_mul(height)?;
    if [c, m, y, k].iter().any(|p| p.len() < plane_size) {
        return None;
    }
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
        format!("{variable_name} (CMYK Composite)"),
        anim_extent,
    ))
}
