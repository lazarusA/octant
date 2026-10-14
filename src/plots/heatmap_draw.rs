//! The heatmap renderer as a `PlotRenderer` and its egui paint callback.

use super::heatmap::HeatmapRenderer;
use std::sync::Arc;

impl super::traits::PlotRenderer for HeatmapRenderer {
    fn update_data(&self, queue: &wgpu::Queue, data: &crate::data::RenderData) {
        if let crate::data::RenderData::Matrix(m) = data {
            self.update_data(queue, &m.values);
        }
    }

    fn paint(
        &self,
        _ui: &mut egui::Ui,
        _rect: egui::Rect,
        _params: &super::traits::PlotRenderParams,
    ) {
        // Concrete painter dispatched via egui callback
    }

    fn inspect_hover(
        &self,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        data: &crate::data::RenderData,
        _params: &super::traits::PlotRenderParams,
    ) -> Option<super::traits::HoverSample> {
        let crate::data::RenderData::Matrix(m) = data else {
            return None;
        };
        if !rect.contains(pointer_pos) || m.width == 0 || m.height == 0 {
            return None;
        }
        let nx = ((pointer_pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 1.0);
        let ny = ((pointer_pos.y - rect.min.y) / rect.height().max(1.0)).clamp(0.0, 1.0);
        let (px, py) = m.grid.find_cell_from_norm(nx, ny, m.width, m.height);
        let val = m.values.get(py * m.width + px).copied().unwrap_or(f32::NAN);
        let (cell_lon, cell_lat) = m.grid.cell_center_lon_lat_rad(px, py, m.width, m.height);
        Some(super::traits::HoverSample {
            cell_x: px,
            cell_y: py,
            value: val,
            coord_lon_lat: Some((cell_lon.to_degrees() as f64, cell_lat.to_degrees() as f64)),
            world_pos: None,
        })
    }
}

/// Paints a heatmap renderer's data: through its colormap, or for a label
/// layer (`labels`) in its label colors.
pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub coord_mode: u32,
    pub labels: Option<super::labels::LabelDraw>,
}

impl eframe::egui_wgpu::CallbackTrait for HeatmapCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        super::colormap_atlas::prepare(device, queue, callback_resources);
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.pan,
            self.zoom,
            self.aspect_scale,
            self.coord_mode,
        );
        if let Some(labels) = &self.labels {
            labels
                .renderer
                .update_params(queue, labels.outline, labels.opacity);
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if !super::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }
        if let Some(labels) = &self.labels {
            labels.renderer.draw(rpass, &self.renderer);
            return;
        }
        if !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }

        rpass.set_pipeline(&self.renderer.render_pipeline);
        self.renderer.bind_quad(rpass);
        self.renderer.draw_quad(rpass);
    }
}
