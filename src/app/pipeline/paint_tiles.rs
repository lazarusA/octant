//! Heatmap tiles resampled from a layer's pyramid to the visible window
//! before painting.

use super::paint_layer::CanvasView;
use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::PlotType;

impl OctantApp {
    /// Resamples each pyramid-backed heatmap layer to the visible window.
    pub(super) fn refresh_resampled_tiles(&mut self, plot_type: PlotType, view: &CanvasView) {
        if plot_type != PlotType::Heatmap {
            return;
        }
        let Some(render_state) = &self.wgpu_render_state else {
            return;
        };
        for layer in self.layers.iter_mut().filter(|l| l.is_drawn()) {
            refresh_layer_tile(layer, &render_state.queue, view);
        }
    }
}

/// Resamples `layer`'s pyramid, when it has one, to the visible window.
fn refresh_layer_tile(layer: &mut Layer, queue: &wgpu::Queue, view: &CanvasView) {
    let Some(renderer) = &layer.renderers.heatmap else {
        return;
    };
    let (Some(_), Some(matrix)) = (&layer.data.pyramid, &layer.data.matrix) else {
        return;
    };
    let (visible_u, visible_v) = crate::data::ViewportResampler::compute_visible_data_bounds(
        view.pan,
        view.zoom,
        view.aspect_scale,
    );
    let (orig_w, orig_h) = layer.data.dimensions_2d();
    let (target_w, target_h) =
        crate::data::ViewportResampler::compute_target_resolution(orig_w, orig_h, 2048);
    if let Some(tile) = layer.data.resampler.resample_if_needed(
        &matrix.values,
        visible_u,
        visible_v,
        target_w,
        target_h,
    ) {
        renderer.update_data_and_dimensions(
            queue,
            tile.values,
            tile.width,
            tile.height,
            tile.tile_bounds,
        );
    }
}
