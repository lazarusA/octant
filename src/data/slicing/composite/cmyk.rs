//! CMYK 4-channel composite slicing to 24-bit TrueColor `MatrixData`.

use crate::data::octant_block::OctantBlock;
use crate::data::{MatrixData, VariableInfo};

use super::stretch::{CompositeStretch, Stretch};
use super::utils::{compute_normalization_scale, linear_to_srgb, pack_rgb};

/// Whether attributes, read through `get`, tag CMYK inks: a `photometric` or `color_space`
/// of `cmyk`, or a `long_name` mentioning CMYK. The slicers and the app's labels both
/// decide through this, so the plot and its readouts agree.
pub fn is_cmyk_attrs<'a>(get: impl Fn(&str) -> Option<&'a str>) -> bool {
    ["photometric", "color_space"]
        .into_iter()
        .any(|key| get(key).is_some_and(|v| v.trim().eq_ignore_ascii_case("cmyk")))
        || get("long_name").is_some_and(|l| l.contains("CMYK"))
}

/// Whether variable `var` holds CMYK inks, by its attributes and `long_name`.
pub fn is_cmyk_var(var: &VariableInfo) -> bool {
    is_cmyk_attrs(|key| match key {
        "long_name" => var.long_name.as_deref(),
        _ => var.attributes.get(key).map(String::as_str),
    })
}

/// Whether `block` holds CMYK inks, bands 0-3, along its channel dimension
/// `c_dim`; a block of other bands is composed as RGB.
pub fn is_cmyk_block(block: &OctantBlock, c_dim: usize) -> bool {
    block.origin.get(c_dim).copied().unwrap_or(0) == 0
        && block.shape.get(c_dim).is_some_and(|&n| n >= 4)
        && is_cmyk_attrs(|key| block.attributes.get(key).map(String::as_str))
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
    let mut stretch = CompositeStretch::default();
    blend_cmyk_planes(
        planes,
        width,
        height,
        &block.variable_name,
        anim_extent,
        &mut stretch,
    )
}

/// Converts four `width * height` C, M, Y, K planes into a TrueColor RGB `MatrixData`, the
/// inks stretched as `stretch` records (else over their values, recorded).
pub fn blend_cmyk_planes(
    inks: [&[f32]; 4],
    width: usize,
    height: usize,
    variable_name: &str,
    anim_extent: usize,
    stretch: &mut CompositeStretch,
) -> Option<MatrixData> {
    let values = blend_cmyk_stretched(inks, width.checked_mul(height)?, stretch)?;
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

/// Packs the first `len` samples of the C, M, Y, K inks into TrueColor RGB, normalizing
/// all inks by their shared range; `None` when an ink is shorter than `len`.
pub fn blend_cmyk(inks: [&[f32]; 4], len: usize) -> Option<Vec<f32>> {
    blend_cmyk_stretched(inks, len, &mut CompositeStretch::default())
}

/// The inks' shared stretch over their values.
fn ink_stretch(inks: [&[f32]; 4]) -> Stretch {
    let (g_min, g_max) = inks.iter().fold((f32::MAX, f32::MIN), |(lo, hi), ink| {
        let (min, max) = crate::utils::compute_finite_min_max(ink);
        (lo.min(min), hi.max(max))
    });
    let is_i8 = (-128.0..0.0).contains(&g_min) && g_max <= 127.0;
    let (scale, offset) = compute_normalization_scale(g_min, g_max, is_i8, 1.0);
    Stretch {
        scale,
        offset,
        is_i8,
    }
}

/// [`blend_cmyk`] with the inks stretched as `stretch` records (else over their values,
/// recorded).
fn blend_cmyk_stretched(
    [c, m, y, k]: [&[f32]; 4],
    len: usize,
    stretch: &mut CompositeStretch,
) -> Option<Vec<f32>> {
    if [c, m, y, k].iter().any(|p| p.len() < len) {
        return None;
    }
    let Stretch {
        scale,
        offset,
        is_i8: is_i8_cmyk,
    } = stretch.inks(|| ink_stretch([c, m, y, k]));
    let mut values = Vec::with_capacity(len);

    for i in 0..len {
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

    Some(values)
}
