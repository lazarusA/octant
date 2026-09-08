//! 2D Viewport Transformation and Heatmap Cell Sampling.

use crate::app::OctantApp;
use crate::data::MatrixData;
use egui::{Pos2, Rect};

/// 2D Viewport Transformation Helper (Aspect-scaling, Zoom, Pan).
#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub rect: Rect,
    pub aspect_scale_x: f32,
    pub aspect_scale_y: f32,
    pub zoom: f32,
    pub gpu_pan_x: f32,
    pub gpu_pan_y: f32,
}

impl Transform2D {
    pub fn from_app(app: &OctantApp, rect: Rect, matrix: &MatrixData) -> Self {
        let (aspect_scale_x, aspect_scale_y) = if app.enforce_data_aspect_ratio {
            let (orig_w, orig_h) = if let Some(pyr) = &app.active_pyramid {
                (pyr.original_width, pyr.original_height)
            } else {
                (matrix.width, matrix.height)
            };
            let data_aspect = (orig_w as f32 / orig_h.max(1) as f32).max(0.001);
            let canvas_aspect = rect.width() / rect.height().max(1.0);
            if canvas_aspect > data_aspect {
                (data_aspect / canvas_aspect, 1.0)
            } else {
                (1.0, canvas_aspect / data_aspect)
            }
        } else {
            (1.0, 1.0)
        };

        let zoom = app.heatmap_zoom;
        let pan = app.heatmap_pan;
        let gpu_pan_x = pan.x / (0.5 * rect.width().max(1.0));
        let gpu_pan_y = -pan.y / (0.5 * rect.height().max(1.0));

        Self {
            rect,
            aspect_scale_x,
            aspect_scale_y,
            zoom,
            gpu_pan_x,
            gpu_pan_y,
        }
    }

    pub fn screen_to_norm(&self, screen_pos: Pos2) -> (f32, f32) {
        let ndc_x = ((screen_pos.x - self.rect.min.x) / self.rect.width().max(1.0)) * 2.0 - 1.0;
        let unpanned_x = (ndc_x - self.gpu_pan_x) / self.zoom.max(0.01);
        let unscaled_x = unpanned_x / self.aspect_scale_x.max(0.001);
        let nx = ((unscaled_x + 1.0) / 2.0).clamp(0.0, 1.0);

        let ndc_y = 1.0 - ((screen_pos.y - self.rect.min.y) / self.rect.height().max(1.0)) * 2.0;
        let unpanned_y = (ndc_y - self.gpu_pan_y) / self.zoom.max(0.01);
        let unscaled_y = unpanned_y / self.aspect_scale_y.max(0.001);
        let ny = ((1.0 - unscaled_y) / 2.0).clamp(0.0, 1.0);

        (nx, ny)
    }

    pub fn norm_to_screen(&self, u: f32, v: f32) -> Pos2 {
        let unscaled_x = u * 2.0 - 1.0;
        let unpanned_x = unscaled_x * self.aspect_scale_x.max(0.001);
        let ndc_x = unpanned_x * self.zoom.max(0.01) + self.gpu_pan_x;
        let target_x = self.rect.min.x + ((ndc_x + 1.0) * 0.5) * self.rect.width();

        let unscaled_y = 1.0 - v * 2.0;
        let unpanned_y = unscaled_y * self.aspect_scale_y.max(0.001);
        let ndc_y = unpanned_y * self.zoom.max(0.01) + self.gpu_pan_y;
        let target_y = self.rect.min.y + ((1.0 - ndc_y) * 0.5) * self.rect.height();

        Pos2::new(target_x, target_y)
    }
}
