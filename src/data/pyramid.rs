// This ports the logic from
// https://github.com/MakieOrg/Makie.jl/blob/master/Makie/src/basic_recipes/datashader.jl
// As of version 0.24.13 (latest on Jul 2026), under the MIT license:
// https://github.com/MakieOrg/Makie.jl/blob/master/LICENSE
//
// Octant's version enhances this by doing downsampling multithreaded using
// rayon, as well as GPU quad mapping. But in other respects the logic is the
// same. So you can find the original implementation in the Makie.jl repository.

//! In-memory 2D matrix pyramid for multi-resolution level-of-detail rendering.

use crate::data::matrix_data::MatrixData;
use rayon::prelude::*;

/// Aggregation operation for downsampling pyramid levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AggregationOp {
    #[default]
    Mean,
    Max,
    Min,
    Nearest,
}

/// A single level in the 2D matrix pyramid.
#[derive(Debug, Clone)]
pub struct PyramidLevel {
    pub level_idx: usize,
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub scale_x: f64,
    pub scale_y: f64,
}

/// In-memory multi-resolution pyramid of a 2D matrix.
#[derive(Debug, Clone)]
pub struct MatrixPyramid {
    pub levels: Vec<PyramidLevel>,
    pub original_width: usize,
    pub original_height: usize,
    pub min_val: f32,
    pub max_val: f32,
    pub dataset_name: String,
}

impl MatrixPyramid {
    /// Builds a full multi-resolution pyramid from raw 2D matrix data.
    pub fn new(
        values: &[f32],
        width: usize,
        height: usize,
        dataset_name: impl Into<String>,
        op: AggregationOp,
        min_resolution: usize,
    ) -> Self {
        let dataset_name = dataset_name.into();
        let (min_val, max_val) = crate::utils::compute_finite_min_max(values);

        let mut levels = Vec::new();

        // Level 0: Full native resolution
        levels.push(PyramidLevel {
            level_idx: 0,
            width,
            height,
            values: values.to_vec(),
            scale_x: 1.0,
            scale_y: 1.0,
        });

        let mut current_w = width;
        let mut current_h = height;
        let min_dim = min_resolution.max(1);

        while current_w > min_dim || current_h > min_dim {
            let next_w = current_w.div_ceil(2);
            let next_h = current_h.div_ceil(2);
            if next_w == current_w && next_h == current_h {
                break;
            }

            let prev_level = levels.last().unwrap();
            let next_values = Self::downsample_2x2(
                &prev_level.values,
                prev_level.width,
                prev_level.height,
                next_w,
                next_h,
                op,
            );

            let scale_x = width as f64 / next_w as f64;
            let scale_y = height as f64 / next_h as f64;

            levels.push(PyramidLevel {
                level_idx: levels.len(),
                width: next_w,
                height: next_h,
                values: next_values,
                scale_x,
                scale_y,
            });

            current_w = next_w;
            current_h = next_h;
        }

        Self {
            levels,
            original_width: width,
            original_height: height,
            min_val,
            max_val,
            dataset_name,
        }
    }

    /// Downsamples a 2D grid by 2x in both dimensions with NaN filtering.
    fn downsample_2x2(
        src: &[f32],
        src_w: usize,
        src_h: usize,
        dst_w: usize,
        dst_h: usize,
        op: AggregationOp,
    ) -> Vec<f32> {
        let mut dst = vec![f32::NAN; dst_w * dst_h];

        dst.par_chunks_mut(dst_w)
            .enumerate()
            .for_each(|(dy, row_out)| {
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
            });

        dst
    }

    /// Selects the best pyramid level for a viewport given visible coordinate spans and canvas resolution.
    pub fn select_level(&self, visible_span_x: f64, target_width: usize) -> usize {
        if self.levels.len() <= 1 || target_width == 0 {
            return 0;
        }

        // Pixels in original data space that correspond to the visible viewport
        let visible_data_px = visible_span_x.abs() * self.original_width as f64;
        let required_sampling_rate = visible_data_px / target_width as f64;

        // Find the coarsest level that still meets or exceeds the required resolution
        let mut best_level = 0;
        for (i, level) in self.levels.iter().enumerate() {
            if level.scale_x <= required_sampling_rate * 1.25 {
                best_level = i;
            } else {
                break;
            }
        }

        best_level.min(self.levels.len() - 1)
    }

    /// Samples a sub-region `[x_min..x_max] x [y_min..y_max]` normalized in `[0.0, 1.0]` at the given target resolution.
    pub fn sample_viewport(
        &self,
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

        let level_idx = self.select_level(span_x, out_w);
        let level = &self.levels[level_idx];

        let mut out_values = vec![f32::NAN; out_w * out_h];

        out_values
            .par_chunks_mut(out_w)
            .enumerate()
            .for_each(|(out_y, row)| {
                let norm_y = y_min + (out_y as f64 / out_h as f64) * span_y;
                let src_y = (norm_y * level.height as f64).floor() as usize;
                let src_y = src_y.min(level.height.saturating_sub(1));

                for (out_x, val) in row.iter_mut().enumerate() {
                    let norm_x = x_min + (out_x as f64 / out_w as f64) * span_x;
                    let src_x = (norm_x * level.width as f64).floor() as usize;
                    let src_x = src_x.min(level.width.saturating_sub(1));

                    *val = level.values[src_y * level.width + src_x];
                }
            });

        MatrixData::new(
            out_w,
            out_h,
            out_values,
            self.min_val,
            self.max_val,
            self.dataset_name.clone(),
            1,
        )
    }
}
