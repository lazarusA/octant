//! egui paint callback for the volume raymarcher.

use super::{VolumeRenderer, VolumeUniformParams};
use std::sync::Arc;

pub struct VolumeCallback {
    pub renderer: Arc<VolumeRenderer>,
    pub params: VolumeUniformParams,
    pub rect: egui::Rect,
    /// Frame resolution relative to the viewport's pixels: below 1 while the
    /// user drags or zooms, 1 once the view settles.
    pub scale: f32,
}

impl eframe::egui_wgpu::CallbackTrait for VolumeCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        crate::plots::colormap_atlas::prepare(device, queue, callback_resources);
        let mut params = self.params;
        params.screen_aspect = crate::plots::common::compute_aspect_ratio(&self.rect);
        // Viewport pixels, rounded as `setup_viewport_and_scissor` does.
        let pixels = self.rect.size() * screen_descriptor.pixels_per_point;
        let scale = self.scale.clamp(0.1, 1.0);
        let size = [pixels.x, pixels.y].map(|p| (p.round() * scale).round().max(1.0) as u32);
        self.renderer
            .render_frame(device, queue, encoder, &params, size, callback_resources);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if !crate::plots::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }
        self.renderer.paint_frame(rpass);
    }
}
