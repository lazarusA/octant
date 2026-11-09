//! The heatmap renderer's egui paint callback.

use super::heatmap::HeatmapRenderer;
use std::sync::Arc;

/// Where a heatmap quad sits on the canvas and what of it is drawn: its
/// pan, zoom and aspect scale, coordinate mode, and the slicer's box.
#[derive(Copy, Clone, Debug)]
pub struct QuadView {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub coord_mode: u32,
    pub slice: super::slice::SliceUniform,
}

impl Default for QuadView {
    fn default() -> Self {
        Self {
            pan: [0.0; 2],
            zoom: 1.0,
            aspect_scale: [1.0; 2],
            coord_mode: 0,
            slice: super::slice::SliceUniform::OFF,
        }
    }
}

/// Paints a heatmap renderer's data: through its colormap, or for a label
/// layer (`labels`) in its label colors.
pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
    pub view: QuadView,
    pub labels: Option<super::labels::LabelDraw>,
    /// A multi-channel composite blended as drawn, in place of the colormap.
    pub channels: Option<super::channels::ChannelDraw>,
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
        self.renderer
            .update_uniforms(queue, &self.color_params, &self.view);
        if let Some(labels) = &self.labels {
            labels
                .renderer
                .update_params(queue, labels.outline, labels.opacity);
        }
        if let Some(channels) = &self.channels {
            channels.renderer.update_params(queue, &channels.params);
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
        if let Some(channels) = &self.channels {
            channels.renderer.draw(rpass, &self.renderer);
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
