//! Canvas viewport, aspect ratio scaling, and GPU uniform transformations.

use crate::app::OctantApp;
use crate::app::state::NavigationState;
use crate::app::state::navigation::QuadWindow;
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
    let (zoom, pan) = (nav.line_zoom, nav.line_pan);
    planar_uniforms(zoom, pan, canvas_rect, [1.0, 1.0], gpu_aspect_scale)
}

fn compute_heatmap_uniforms(
    nav: &NavigationState,
    canvas_rect: egui::Rect,
    gpu_aspect_scale: [f32; 2],
) -> ViewportUniforms {
    let (zoom, pan) = (nav.heatmap_zoom, nav.heatmap_pan);
    planar_uniforms(zoom, pan, canvas_rect, gpu_aspect_scale, gpu_aspect_scale)
}

/// A 2D plot's rectangle on the canvas, `rect_scale` of the canvas at `zoom`
/// and moved by `pan` (points), and the matching GPU pan and zoom.
fn planar_uniforms(
    zoom: f32,
    pan: egui::Vec2,
    canvas_rect: egui::Rect,
    [scale_x, scale_y]: [f32; 2],
    gpu_aspect_scale: [f32; 2],
) -> ViewportUniforms {
    let size = egui::vec2(
        canvas_rect.width() * scale_x * zoom,
        canvas_rect.height() * scale_y * zoom,
    );
    let rect = egui::Rect::from_center_size(canvas_rect.center() + pan, size);
    let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
    let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));

    ViewportUniforms {
        plot_rect: rect,
        gpu_pan: [gpu_pan_x, gpu_pan_y],
        gpu_zoom: zoom,
        gpu_aspect_scale,
    }
}

/// The heatmap's zoom and pan (screen points) showing `window` of the
/// plotted quad on a canvas of `canvas` points, the inverse of
/// `compute_heatmap_uniforms` and `compute_visible_data_bounds`: the window
/// is contained (a canvas of another shape shows more around it), centred,
/// its zoom within `(min, max)`.
pub fn heatmap_view_for(
    ((u0, u1), (v0, v1)): QuadWindow,
    canvas: egui::Vec2,
    [ax, ay]: [f32; 2],
    (min, max): (f32, f32),
) -> (f32, egui::Vec2) {
    let (ax, ay) = (f64::from(ax.max(1e-4)), f64::from(ay.max(1e-4)));
    let (du, dv) = ((u1 - u0).abs().max(1e-9), (v1 - v0).abs().max(1e-9));
    let zoom = (1.0 / (ax * du)).min(1.0 / (ay * dv)) as f32;
    let zoom = zoom.clamp(min, max.max(min));
    let z = f64::from(zoom);
    // Model positions of the window's centre: x grows right, y up.
    let (xc, yc) = (u0 + u1 - 1.0, 1.0 - (v0 + v1));
    let (gpu_x, gpu_y) = (-xc * ax * z, -yc * ay * z);
    let pan = egui::vec2(
        (gpu_x * 0.5 * f64::from(canvas.x.max(1.0))) as f32,
        (-gpu_y * 0.5 * f64::from(canvas.y.max(1.0))) as f32,
    );
    (zoom, pan)
}

impl OctantApp {
    /// Shows the pending view (`SetView`) once the heatmap draws the plot
    /// it was set for; a view whose plot was replaced is dropped.
    pub fn apply_pending_view(&mut self, canvas_rect: egui::Rect, plot_type: PlotType) {
        let Some(view) = self.nav.pending_view else {
            return;
        };
        let plotted = self.plotted();
        if view.plot != (plotted.metadata_generation, plotted.variable_idx) {
            self.nav.pending_view = None;
            return;
        }
        if plot_type != PlotType::Heatmap || !self.has_plot() {
            return;
        }
        let aspect = self.compute_aspect_scale(canvas_rect.size());
        let min = crate::app::controllers::controller_for(plot_type).min_zoom();
        let limits = (min, self.nav.heatmap_max_zoom);
        let (zoom, pan) = heatmap_view_for(view.window, canvas_rect.size(), aspect, limits);
        self.nav.heatmap_zoom = zoom;
        self.nav.heatmap_pan = pan;
        self.nav.pending_view = None;
    }
}

#[cfg(test)]
#[path = "viewport_tests.rs"]
mod tests;
