//! Direct sample blitting from decoded TIFF chunk buffers to output slices.

use async_tiff::tags::SampleFormat;

use crate::data::calibration::transform::matches_fill;

/// Sliced hyperslab window coordinate ranges.
#[derive(Debug, Clone, Copy)]
pub struct ReadWindow {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub nodata_val: Option<f64>,
}

/// Blit decoded raw bytes from a chunk (tile or strip) directly into destination hyperslab slices.
#[allow(clippy::too_many_arguments)]
pub fn blit_chunk_to_window(
    raw: &[u8],
    chunk_w: usize,
    chunk_h: usize,
    samples: usize,
    is_planar: bool,
    origin_x: usize,
    origin_y: usize,
    sample_fmt: SampleFormat,
    bits: u16,
    is_white_zero: bool,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) {
    let out_w = win.col_end.saturating_sub(win.col_start).max(1);
    let r_min = win
        .row_start
        .max(origin_y)
        .min(win.row_end.min(origin_y + chunk_h));
    let r_max = win.row_end.min(origin_y + chunk_h);
    let c_min = win
        .col_start
        .max(origin_x)
        .min(win.col_end.min(origin_x + chunk_w));
    let c_max = win.col_end.min(origin_x + chunk_w);

    if r_min >= r_max || c_min >= c_max {
        return;
    }

    let plane_size = chunk_w * chunk_h;
    for r in r_min..r_max {
        let local_r = r - origin_y;
        let dst_row_base = (r - win.row_start) * out_w;

        for c in c_min..c_max {
            let local_c = c - origin_x;
            let dst_idx = dst_row_base + (c - win.col_start);

            for (i, &band) in target_bands.iter().enumerate() {
                if let Some(out_slice) = out_slices.get_mut(i) {
                    let linear_idx = if is_planar {
                        band * plane_size + local_r * chunk_w + local_c
                    } else {
                        local_r * (chunk_w * samples) + local_c * samples + band
                    };
                    let val = read_sample(
                        raw,
                        chunk_w,
                        samples,
                        is_planar,
                        linear_idx,
                        local_r,
                        local_c,
                        band,
                        sample_fmt,
                        bits,
                        is_white_zero,
                    );
                    out_slice[dst_idx] = if is_nodata(val, win.nodata_val) {
                        f32::NAN
                    } else {
                        val
                    };
                }
            }
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn read_sample(
    raw: &[u8],
    w: usize,
    samples: usize,
    is_planar: bool,
    idx: usize,
    r: usize,
    c: usize,
    band: usize,
    fmt: SampleFormat,
    bits: u16,
    is_white_zero: bool,
) -> f32 {
    match (fmt, bits) {
        (SampleFormat::Uint, 8) => {
            let b = raw.get(idx).copied().unwrap_or(0);
            if is_white_zero {
                (255 - b) as f32
            } else {
                b as f32
            }
        }
        (SampleFormat::Int, 8) => raw.get(idx).map(|&b| b as i8 as f32).unwrap_or(f32::NAN),
        (SampleFormat::Uint, 16) => read_array::<2>(raw, idx * 2)
            .map(u16::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Int, 16) => read_array::<2>(raw, idx * 2)
            .map(i16::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Float, 16) => read_array::<2>(raw, idx * 2)
            .map(u16::from_ne_bytes)
            .map(|v| half::f16::from_bits(v).to_f32())
            .unwrap_or(f32::NAN),
        (SampleFormat::Float, 32) => read_array::<4>(raw, idx * 4)
            .map(f32::from_ne_bytes)
            .unwrap_or(f32::NAN),
        (SampleFormat::Uint, 32) => read_array::<4>(raw, idx * 4)
            .map(u32::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Int, 32) => read_array::<4>(raw, idx * 4)
            .map(i32::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Float, 64) => read_array::<8>(raw, idx * 8)
            .map(f64::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Uint, 64) => read_array::<8>(raw, idx * 8)
            .map(u64::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Int, 64) => read_array::<8>(raw, idx * 8)
            .map(i64::from_ne_bytes)
            .map(|v| v as f32)
            .unwrap_or(f32::NAN),
        (SampleFormat::Uint, 1) => {
            read_bit(raw, w, samples, is_planar, (r, c, band), is_white_zero)
        }
        (SampleFormat::Uint, 4) => read_nibble(raw, w, samples, is_planar, (r, c, band)),
        (SampleFormat::Uint, 12) => read_12bit(raw, w, samples, is_planar, (r, c, band)),
        _ => raw.get(idx).map(|&b| b as f32).unwrap_or(f32::NAN),
    }
}

#[inline(always)]
fn read_bit(
    raw: &[u8],
    w: usize,
    samples: usize,
    is_planar: bool,
    (r, c, band): (usize, usize, usize),
    is_white_zero: bool,
) -> f32 {
    let row_bytes = (w * if is_planar { 1 } else { samples }).div_ceil(8);
    let s_in_row = if is_planar { c } else { c * samples + band };
    let bit = raw
        .get(r * row_bytes + s_in_row / 8)
        .map(|&b| (b >> (7 - (s_in_row % 8))) & 1)
        .unwrap_or(0);
    if is_white_zero {
        if bit == 0 { 1.0 } else { 0.0 }
    } else {
        bit as f32
    }
}

#[inline(always)]
fn read_nibble(
    raw: &[u8],
    w: usize,
    samples: usize,
    is_planar: bool,
    (r, c, band): (usize, usize, usize),
) -> f32 {
    let row_bytes = (w * if is_planar { 1 } else { samples }).div_ceil(2);
    let s_in_row = if is_planar { c } else { c * samples + band };
    let byte = raw.get(r * row_bytes + s_in_row / 2).copied().unwrap_or(0);
    let nibble = if s_in_row % 2 == 0 {
        byte >> 4
    } else {
        byte & 0x0F
    };
    nibble as f32
}

#[inline(always)]
fn read_12bit(
    raw: &[u8],
    w: usize,
    samples: usize,
    is_planar: bool,
    (r, c, band): (usize, usize, usize),
) -> f32 {
    let row_bytes = ((w * if is_planar { 1 } else { samples }) * 12).div_ceil(8);
    let s_in_row = if is_planar { c } else { c * samples + band };
    let byte_idx = r * row_bytes + (s_in_row / 2) * 3;
    if s_in_row % 2 == 0 {
        let (b0, b1) = (
            raw.get(byte_idx).copied().unwrap_or(0) as u16,
            raw.get(byte_idx + 1).copied().unwrap_or(0) as u16,
        );
        ((b0 << 4) | (b1 >> 4)) as f32
    } else {
        let (b1, b2) = (
            raw.get(byte_idx + 1).copied().unwrap_or(0) as u16,
            raw.get(byte_idx + 2).copied().unwrap_or(0) as u16,
        );
        (((b1 & 0x0F) << 8) | b2) as f32
    }
}

#[inline(always)]
fn read_array<const N: usize>(raw: &[u8], offset: usize) -> Option<[u8; N]> {
    raw.get(offset..offset + N).and_then(|s| s.try_into().ok())
}

/// Whether a sample is missing: NaN, or the nodata value (only NaN samples when nodata
/// is NaN).
#[inline]
pub fn is_nodata(val: f32, nodata: Option<f64>) -> bool {
    val.is_nan() || nodata.is_some_and(|nd| matches_fill(f64::from(val), nd))
}
