//! One layer's GPU paint callback for the canvas plot type, and the heatmap
//! tiles resampled to the visible window before painting.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::PlotType;

/// Where the canvas draws this frame.
#[derive(Clone, Copy)]
pub struct CanvasView {
    pub canvas_rect: egui::Rect,
    pub plot_rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
}

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

    /// Adds `layer`'s paint callback for `plot_type`, when its renderer exists.
    pub(super) fn paint_layer(
        &self,
        ui: &egui::Ui,
        layer: &Layer,
        plot_type: PlotType,
        view: &CanvasView,
    ) {
        match plot_type {
            PlotType::Line => self.paint_line(ui, layer, view),
            PlotType::Sphere | PlotType::Surface => self.paint_mesh(ui, layer, plot_type, view),
            PlotType::Volume => self.paint_volume(ui, layer, view),
            PlotType::PointCloud => self.paint_point_cloud(ui, layer, view),
            PlotType::Heatmap => self.paint_heatmap(ui, layer, view),
        }
    }

    fn paint_line(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(line_renderer) = &layer.renderers.line else {
            return;
        };
        let color_params = self.get_color_params(layer);
        let payload = self.line_payload();
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.canvas_rect,
            crate::plots::LineCallback {
                renderer: line_renderer.clone(),
                color_params,
                line_color: self.line_color,
                use_custom_color: self.line_use_custom_color,
                show_lines: self.line_show_lines,
                show_points: self.line_show_points,
                point_size: self.line_point_size,
                rect: view.canvas_rect,
                profile_values: payload.values,
                profile_length: payload.profile_length,
                drawn_lines: payload.drawn_lines,
                line_count: payload.line_count,
                line_mode: if self.line_plot_all_series { 1 } else { 0 },
                pan: view.pan,
                zoom: view.zoom,
            },
        );
        ui.painter().add(callback);
    }

    fn paint_mesh(&self, ui: &egui::Ui, layer: &Layer, plot_type: PlotType, view: &CanvasView) {
        let (renderer, mode, displacement, cube_mode_idx) = match plot_type {
            PlotType::Sphere => (
                &layer.renderers.sphere,
                self.sphere_mode,
                self.sphere_displacement_strength,
                3,
            ),
            _ => (
                &layer.renderers.surface,
                self.surface_mode,
                self.surface_displacement_strength,
                2,
            ),
        };
        let Some(renderer) = renderer else {
            return;
        };
        let aspect_ratio = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_mesh_3d_uniform_params(layer, mode, displacement, aspect_ratio);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::Mesh3DCallback {
                renderer: renderer.clone(),
                params,
                cube_mode_idx,
                rect: view.plot_rect,
                transparency: self.transparency_mode(layer),
            },
        );
        ui.painter().add(callback);
    }

    fn paint_volume(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(volume_renderer) = &layer.renderers.volume else {
            return;
        };
        let screen_aspect = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_volume_uniform_params(layer, screen_aspect);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::VolumeCallback {
                renderer: volume_renderer.clone(),
                params,
                rect: view.plot_rect,
                // Half resolution while rotating or zooming; the
                // frame after the input stops renders in full.
                scale: if self.view_interacting { 0.5 } else { 1.0 },
            },
        );
        ui.painter().add(callback);
    }

    fn paint_point_cloud(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(point_cloud_renderer) = &layer.renderers.point_cloud else {
            return;
        };
        let screen_aspect = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_point_cloud_uniform_params(layer, screen_aspect);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::PointCloudCallback {
                renderer: point_cloud_renderer.clone(),
                params,
                rect: view.plot_rect,
                transparency: self.transparency_mode(layer),
            },
        );
        ui.painter().add(callback);
    }

    fn paint_heatmap(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(renderer) = &layer.renderers.heatmap else {
            return;
        };
        let coord_mode = layer
            .data
            .matrix
            .as_ref()
            .map_or(0, |m| m.grid.render_coord_mode());
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.canvas_rect,
            crate::plots::MatrixCallback {
                renderer: renderer.clone(),
                color_params: self.get_color_params(layer),
                rect: view.canvas_rect,
                pan: view.pan,
                zoom: view.zoom,
                aspect_scale: view.aspect_scale,
                coord_mode,
            },
        );
        ui.painter().add(callback);
    }
}

/// Resamples `layer`'s pyramid, when it has one, to the visible window.
fn refresh_layer_tile(layer: &mut Layer, queue: &wgpu::Queue, view: &CanvasView) {
    let Some(renderer) = &layer.renderers.heatmap else {
        return;
    };
    if layer.data.pyramid.is_none() {
        return;
    }
    let (visible_u, visible_v) = crate::data::ViewportResampler::compute_visible_data_bounds(
        view.pan,
        view.zoom,
        view.aspect_scale,
    );
    let (orig_w, orig_h) = layer.data.dimensions_2d();
    let (target_w, target_h) =
        crate::data::ViewportResampler::compute_target_resolution(orig_w, orig_h, 2048);
    if let Some(tile) = layer
        .data
        .resampler
        .resample_if_needed(visible_u, visible_v, target_w, target_h)
    {
        renderer.update_data_and_dimensions(
            queue,
            &tile.data.values,
            tile.data.width,
            tile.data.height,
            tile.tile_bounds,
        );
    }
}
