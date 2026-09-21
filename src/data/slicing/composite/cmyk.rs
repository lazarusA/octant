//! CMYK 4-channel composite slicing to 24-bit TrueColor `MatrixData`.

use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;

use super::utils::{compute_normalization_scale, linear_to_srgb, pack_rgb};

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
