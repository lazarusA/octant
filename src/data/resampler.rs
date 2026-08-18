//! Viewport-aware resampler that coordinates interactive pan/zoom with MatrixPyramid.

use std::sync::Arc;
use crate::data::matrix_data::MatrixData;
use crate::data::pyramid::MatrixPyramid;

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

    /// Resamples the visible viewport if the request has changed meaningfully.
    pub fn resample_if_needed(
        &mut self,
        x_range: (f64, f64),
        y_range: (f64, f64),
        target_width: usize,
        target_height: usize,
    ) -> Option<MatrixData> {
        let pyramid = self.pyramid.as_ref()?;

        let target_w = target_width.clamp(16, self.max_resolution);
        let target_h = target_height.clamp(16, self.max_resolution);

        let span_x = (x_range.1 - x_range.0).abs().max(1e-6);
        let level_idx = pyramid.select_level(span_x, target_w);

        let req = ViewportRequest {
            x_range,
            y_range,
            target_width: target_w,
            target_height: target_h,
            level_idx,
        };

        if let Some(last) = &self.last_request {
            let last_span_x = (last.x_range.1 - last.x_range.0).abs().max(1e-6);
            let zoom_ratio = span_x / last_span_x;

            let x_diff = (req.x_range.0 - last.x_range.0).abs() + (req.x_range.1 - last.x_range.1).abs();
            let y_diff = (req.y_range.0 - last.y_range.0).abs() + (req.y_range.1 - last.y_range.1).abs();
            let w_diff = req.target_width.abs_diff(last.target_width);
            let h_diff = req.target_height.abs_diff(last.target_height);

            // If same pyramid LOD level, and zoom changed by < 30%, and pan moved by < 8% of span, skip re-upload
            if req.level_idx == last.level_idx
                && (0.75..=1.33).contains(&zoom_ratio)
                && x_diff < span_x * 0.08
                && y_diff < (y_range.1 - y_range.0).abs().max(1e-6) * 0.08
                && w_diff < 128
                && h_diff < 128
            {
                return None;
            }
        }

        let sampled = pyramid.sample_viewport(x_range, y_range, (target_w, target_h));
        self.last_request = Some(req);

        Some(sampled)
    }
}
