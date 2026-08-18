//! Viewport-aware resampler that coordinates interactive pan/zoom with MatrixPyramid.

use crate::data::matrix_data::MatrixData;
use crate::data::pyramid::MatrixPyramid;
use std::sync::Arc;

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

    /// Resamples the full-domain texture at the optimal pyramid LOD level for the visible span.
    pub fn resample_if_needed(
        &mut self,
        visible_span_x: f64,
        target_width: usize,
        target_height: usize,
    ) -> Option<MatrixData> {
        let pyramid = self.pyramid.as_ref()?;

        let target_w = target_width.clamp(16, self.max_resolution);
        let target_h = target_height.clamp(16, self.max_resolution);

        let span_x = visible_span_x.abs().max(1e-6);
        let level_idx = pyramid.select_level(span_x, target_w);

        let req = ViewportRequest {
            x_range: (0.0, 1.0),
            y_range: (0.0, 1.0),
            target_width: target_w,
            target_height: target_h,
            level_idx,
        };

        if let Some(last) = &self.last_request
            && req.level_idx == last.level_idx
            && req.target_width == last.target_width
            && req.target_height == last.target_height
        {
            return None;
        }

        let sampled = pyramid.sample_viewport((0.0, 1.0), (0.0, 1.0), (target_w, target_h));
        self.last_request = Some(req);

        Some(sampled)
    }
}
