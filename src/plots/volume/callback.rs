//! egui paint callback for the volume raymarcher.

use super::{VolumeRenderer, VolumeUniformParams};
use std::sync::Arc;

pub struct VolumeCallback {
    pub renderer: Arc<VolumeRenderer>,
    pub params: VolumeUniformParams,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for VolumeCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut params = self.params;
        params.screen_aspect = crate::plots::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(queue, &params);
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
        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
        // The bounding box's 36 vertices come from `vs_main`'s vertex index.
        rpass.draw(0..36, 0..1);
    }
}
