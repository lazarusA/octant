//! Canvas viewport, aspect ratio scaling, and GPU uniform transformations.

use crate::app::OctantApp;
use crate::app::state::NavigationState;
use crate::plots::PlotType;

/// Uniforms and screen-space rect computed for canvas rendering.
#[derive(Debug, Clone, Copy)]
pub struct ViewportUniforms {
    pub plot_rect: egui::Rect,
    pub gpu_pan: [f32; 2],
    pub gpu_zoom: f32,
    pub gpu_aspect_scale: [f32; 2],
}

/// Computes screen-space transformed plot rect and GPU pan/zoom uniforms.
pub fn compute_viewport_uniforms(
    app: &OctantApp,
    canvas_rect: egui::Rect,
    canvas_plot_type: PlotType,
) -> ViewportUniforms {
    let gpu_aspect_scale = app.compute_aspect_scale(canvas_rect.size());

    if canvas_plot_type.is_3d() {
        ViewportUniforms {
            plot_rect: canvas_rect,
            gpu_pan: [0.0, 0.0],
            gpu_zoom: 1.0,
            gpu_aspect_scale,
        }
    } else if canvas_plot_type == PlotType::Line {
        compute_line_uniforms(&app.nav, canvas_rect, gpu_aspect_scale)
    } else {
        compute_heatmap_uniforms(&app.nav, canvas_rect, gpu_aspect_scale)
    }
}

fn compute_line_uniforms(
    nav: &NavigationState,
    canvas_rect: egui::Rect,
    gpu_aspect_scale: [f32; 2],
) -> ViewportUniforms {
    let zoom = nav.line_zoom;
    let pan = nav.line_pan;
    let scaled_size = canvas_rect.size() * zoom;
    let scaled_center = canvas_rect.center() + pan;
    let rect = egui::Rect::from_center_size(scaled_center, scaled_size);
    let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
    let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));

    ViewportUniforms {
        plot_rect: rect,
        gpu_pan: [gpu_pan_x, gpu_pan_y],
        gpu_zoom: zoom,
        gpu_aspect_scale,
    }
}

fn compute_heatmap_uniforms(
    nav: &NavigationState,
    canvas_rect: egui::Rect,
    gpu_aspect_scale: [f32; 2],
) -> ViewportUniforms {
    let [aspect_scale_x, aspect_scale_y] = gpu_aspect_scale;
    let zoom = nav.heatmap_zoom;
    let pan = nav.heatmap_pan;
    let plot_w = canvas_rect.width() * aspect_scale_x * zoom;
    let plot_h = canvas_rect.height() * aspect_scale_y * zoom;
    let scaled_center = canvas_rect.center() + pan;
    let rect = egui::Rect::from_center_size(scaled_center, egui::vec2(plot_w, plot_h));

    let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
    let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));

    ViewportUniforms {
        plot_rect: rect,
        gpu_pan: [gpu_pan_x, gpu_pan_y],
        gpu_zoom: zoom,
        gpu_aspect_scale,
    }
}
