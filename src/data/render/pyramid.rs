// This ports the logic from
// https://github.com/MakieOrg/Makie.jl/blob/master/Makie/src/basic_recipes/datashader.jl
// As of version 0.24.13 (latest on Jul 2026), under the MIT license:
// https://github.com/MakieOrg/Makie.jl/blob/master/LICENSE
//
// Octant's version enhances this by doing downsampling multithreaded using
// rayon, as well as GPU quad mapping. But in other respects the logic is the
// same. So you can find the original implementation in the Makie.jl repository.

//! In-memory 2D matrix pyramid for multi-resolution level-of-detail rendering.

use super::matrix::MatrixData;
use super::pyramid_downsample;

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

impl PyramidLevel {
    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>() + std::mem::size_of::<Self>()
    }
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
    pub fn bytes_size(&self) -> usize {
        self.levels.iter().map(|l| l.bytes_size()).sum::<usize>()
            + self.dataset_name.len()
            + std::mem::size_of::<Self>()
    }

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

            let Some(prev_level) = levels.last() else {
                break;
            };
            let next_values = pyramid_downsample::downsample_2x2(
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
    pub fn downsample_2x2(
        src: &[f32],
        src_w: usize,
        src_h: usize,
        dst_w: usize,
        dst_h: usize,
        op: AggregationOp,
    ) -> Vec<f32> {
        pyramid_downsample::downsample_2x2(src, src_w, src_h, dst_w, dst_h, op)
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
        pyramid_downsample::sample_viewport(self, x_range, y_range, target_res)
    }
}
