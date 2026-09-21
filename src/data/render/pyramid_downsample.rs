//! Downsampling and viewport sampling algorithms for 2D matrix pyramids.

use super::matrix::MatrixData;
use super::pyramid::{AggregationOp, MatrixPyramid, PyramidLevel};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

/// Downsamples a 2D grid by 2x in both dimensions with NaN filtering.
pub fn downsample_2x2(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    op: AggregationOp,
) -> Vec<f32> {
    let mut dst = vec![f32::NAN; dst_w * dst_h];

    let process_row = |(dy, row_out): (usize, &mut [f32])| {
        let sy0 = dy * 2;
        let sy1 = (sy0 + 1).min(src_h - 1);

        for (dx, out_val) in row_out.iter_mut().enumerate() {
            let sx0 = dx * 2;
            let sx1 = (sx0 + 1).min(src_w - 1);

            let p00 = src[sy0 * src_w + sx0];
            let p10 = src[sy0 * src_w + sx1];
            let p01 = src[sy1 * src_w + sx0];
            let p11 = src[sy1 * src_w + sx1];

            let samples = [p00, p10, p01, p11];

            *out_val = match op {
                AggregationOp::Nearest => p00,
                AggregationOp::Mean => {
                    let mut sum = 0.0f32;
                    let mut count = 0usize;
                    for &s in &samples {
                        if s.is_finite() {
                            sum += s;
                            count += 1;
                        }
                    }
                    if count > 0 {
                        sum / count as f32
                    } else {
                        f32::NAN
                    }
                }
                AggregationOp::Max => {
                    let mut max = f32::NEG_INFINITY;
                    let mut found = false;
                    for &s in &samples {
                        if s.is_finite() {
                            max = max.max(s);
                            found = true;
                        }
                    }
                    if found { max } else { f32::NAN }
                }
                AggregationOp::Min => {
                    let mut min = f32::INFINITY;
                    let mut found = false;
                    for &s in &samples {
                        if s.is_finite() {
                            min = min.min(s);
                            found = true;
                        }
                    }
                    if found { min } else { f32::NAN }
                }
            };
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    dst.par_chunks_mut(dst_w).enumerate().for_each(process_row);

    #[cfg(target_arch = "wasm32")]
    dst.chunks_mut(dst_w).enumerate().for_each(process_row);

    dst
}

/// Samples a sub-region `[x_min..x_max] x [y_min..y_max]` normalized in `[0.0, 1.0]` at the given target resolution.
pub fn sample_viewport(
    pyramid: &MatrixPyramid,
    x_range: (f64, f64),
    y_range: (f64, f64),
    target_res: (usize, usize),
) -> MatrixData {
    let (x_min, x_max) = (
        x_range.0.min(x_range.1).clamp(0.0, 1.0),
        x_range.0.max(x_range.1).clamp(0.0, 1.0),
    );
    let (y_min, y_max) = (
        y_range.0.min(y_range.1).clamp(0.0, 1.0),
        y_range.0.max(y_range.1).clamp(0.0, 1.0),
    );

    let span_x = (x_max - x_min).max(1e-6);
    let span_y = (y_max - y_min).max(1e-6);

    let out_w = target_res.0.clamp(2, 2048);
    let out_h = target_res.1.clamp(2, 2048);

    let level_idx = pyramid.select_level(span_x, out_w);
    let level: &PyramidLevel = &pyramid.levels[level_idx];

    let mut out_values = vec![f32::NAN; out_w * out_h];

    let process_sample_row = |(out_y, row): (usize, &mut [f32])| {
        let norm_y = y_min + (out_y as f64 / out_h as f64) * span_y;
        let src_y = (norm_y * level.height as f64).floor() as usize;
        let src_y = src_y.min(level.height.saturating_sub(1));

        for (out_x, val) in row.iter_mut().enumerate() {
            let norm_x = x_min + (out_x as f64 / out_w as f64) * span_x;
            let src_x = (norm_x * level.width as f64).floor() as usize;
            let src_x = src_x.min(level.width.saturating_sub(1));

            *val = level.values[src_y * level.width + src_x];
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    out_values
        .par_chunks_mut(out_w)
        .enumerate()
        .for_each(process_sample_row);

    #[cfg(target_arch = "wasm32")]
    out_values
        .chunks_mut(out_w)
        .enumerate()
        .for_each(process_sample_row);

    MatrixData::new(
        out_w,
        out_h,
        out_values,
        pyramid.min_val,
        pyramid.max_val,
        pyramid.dataset_name.clone(),
        1,
    )
}
