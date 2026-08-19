//! Viewport-aware resampler that coordinates interactive pan/zoom with MatrixPyramid.

use crate::data::matrix_data::MatrixData;
use crate::data::pyramid::MatrixPyramid;
use std::sync::Arc;

/// A sampled viewport tile along with its normalized bounds in the source dataset.
#[derive(Debug, Clone)]
pub struct ResampledTile {
    pub data: MatrixData,
    pub tile_bounds: [f32; 4], // [u_min, v_min, u_max, v_max]
}

/// Request for a viewport sample.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportRequest {
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub target_width: usize,
    pub target_height: usize,
    pub level_idx: usize,
}

/// Viewport resampler managing level-of-detail extraction.
#[derive(Debug, Clone)]
pub struct ViewportResampler {
    pub pyramid: Option<Arc<MatrixPyramid>>,
    pub last_request: Option<ViewportRequest>,
    pub max_resolution: usize,
}

impl Default for ViewportResampler {
    fn default() -> Self {
        Self {
            pyramid: None,
            last_request: None,
            max_resolution: 2048,
        }
    }
}

impl ViewportResampler {
    pub fn new(pyramid: Option<Arc<MatrixPyramid>>) -> Self {
        Self {
            pyramid,
            last_request: None,
            max_resolution: 2048,
        }
    }

    pub fn set_pyramid(&mut self, pyramid: Option<Arc<MatrixPyramid>>) {
        self.pyramid = pyramid;
        self.last_request = None;
    }

    /// Computes target output resolution preserving original aspect ratio up to `max_dimension`.
    pub fn compute_target_resolution(
        orig_w: usize,
        orig_h: usize,
        max_dimension: usize,
    ) -> (usize, usize) {
        let max_dim = max_dimension.max(16);
        if orig_w == 0 || orig_h == 0 {
            return (max_dim, max_dim);
        }
        if orig_w >= orig_h {
            let w = orig_w.min(max_dim);
            let h = ((w as f64 * orig_h as f64) / orig_w as f64)
                .round()
                .max(2.0) as usize;
            (w, h)
        } else {
            let h = orig_h.min(max_dim);
            let w = ((h as f64 * orig_w as f64) / orig_h as f64)
                .round()
                .max(2.0) as usize;
            (w, h)
        }
    }

    /// Computes normalized `[0.0, 1.0]` visible data bounds from pan, zoom, and aspect scale.
    pub fn compute_visible_data_bounds(
        pan: [f32; 2],
        zoom: f32,
        aspect_scale: [f32; 2],
    ) -> ((f64, f64), (f64, f64)) {
        let zoom = (zoom as f64).max(0.001);
        let pan_x = pan[0] as f64;
        let pan_y = pan[1] as f64;
        let aspect_x = (aspect_scale[0] as f64).max(1e-4);
        let aspect_y = (aspect_scale[1] as f64).max(1e-4);

        // In shader: screen_ndc = model_pos * aspect * zoom + pan
        // model_pos = (screen_ndc - pan) / (aspect * zoom)
        // NDC is in [-1.0, 1.0]
        let x_left = (-1.0 - pan_x) / (aspect_x * zoom);
        let x_right = (1.0 - pan_x) / (aspect_x * zoom);

        let y_top = (1.0 - pan_y) / (aspect_y * zoom);
        let y_bottom = (-1.0 - pan_y) / (aspect_y * zoom);

        // Convert model_pos in [-1.0, 1.0] to normalized UV in [0.0, 1.0]
        let u_min = (x_left + 1.0) * 0.5;
        let u_max = (x_right + 1.0) * 0.5;

        let v_min = (1.0 - y_top) * 0.5;
        let v_max = (1.0 - y_bottom) * 0.5;

        ((u_min, u_max), (v_min, v_max))
    }

    /// Resamples the visible viewport with a buffer margin at the optimal pyramid LOD level.
    pub fn resample_if_needed(
        &mut self,
        visible_u: (f64, f64),
        visible_v: (f64, f64),
        target_width: usize,
        target_height: usize,
    ) -> Option<ResampledTile> {
        let pyramid = self.pyramid.as_ref()?;

        let target_w = target_width.clamp(16, self.max_resolution);
        let target_h = target_height.clamp(16, self.max_resolution);

        let u_min_vis = visible_u.0.min(visible_u.1).clamp(0.0, 1.0);
        let u_max_vis = visible_u.0.max(visible_u.1).clamp(0.0, 1.0);
        let v_min_vis = visible_v.0.min(visible_v.1).clamp(0.0, 1.0);
        let v_max_vis = visible_v.0.max(visible_v.1).clamp(0.0, 1.0);

        let span_u = (u_max_vis - u_min_vis).max(1e-6);
        let span_v = (v_max_vis - v_min_vis).max(1e-6);

        // Pre-buffer a 20% margin around visible viewport
        let margin_u = span_u * 0.20;
        let margin_v = span_v * 0.20;
        let tile_u_min = (u_min_vis - margin_u).clamp(0.0, 1.0);
        let tile_u_max = (u_max_vis + margin_u).clamp(0.0, 1.0);
        let tile_v_min = (v_min_vis - margin_v).clamp(0.0, 1.0);
        let tile_v_max = (v_max_vis + margin_v).clamp(0.0, 1.0);

        let level_idx = pyramid.select_level(span_u, target_w);

        let req = ViewportRequest {
            x_range: (tile_u_min, tile_u_max),
            y_range: (tile_v_min, tile_v_max),
            target_width: target_w,
            target_height: target_h,
            level_idx,
        };

        if let Some(last) = &self.last_request {
            let last_span_u = (last.x_range.1 - last.x_range.0).abs().max(1e-6);
            let zoom_ratio = (tile_u_max - tile_u_min) / last_span_u;

            let is_contained = u_min_vis >= last.x_range.0
                && u_max_vis <= last.x_range.1
                && v_min_vis >= last.y_range.0
                && v_max_vis <= last.y_range.1;

            if is_contained
                && req.level_idx == last.level_idx
                && (0.75..=1.33).contains(&zoom_ratio)
            {
                return None;
            }
        }

        let sampled = pyramid.sample_viewport(
            (tile_u_min, tile_u_max),
            (tile_v_min, tile_v_max),
            (target_w, target_h),
        );
        let tile_bounds = [
            tile_u_min as f32,
            tile_v_min as f32,
            tile_u_max as f32,
            tile_v_max as f32,
        ];
        self.last_request = Some(req);

        Some(ResampledTile {
            data: sampled,
            tile_bounds,
        })
    }
}
