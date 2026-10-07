//! Point cloud draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::oit::{Transparency, Variant, build_pipeline};
use super::point_cloud::{PointCloudRenderer, PointCloudUniformParams, PointCloudVertex};
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl PointCloudRenderer {
    /// Binds group 0 and the billboard template and draws every point.
    fn draw_points(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(
            0..self.index_count,
            0,
            0..self.instance_count.load(Ordering::Relaxed),
        );
    }

    /// The points' pipeline for an OIT [`Variant`].
    pub(super) fn oit_pipeline(
        &self,
        device: &wgpu::Device,
        variant: &Variant<'_>,
    ) -> wgpu::RenderPipeline {
        build_pipeline(
            device,
            &self.pipeline_layout,
            &self.shader,
            &[Some(PointCloudVertex::desc())],
            None,
            variant,
        )
    }
}

pub struct PointCloudCallback {
    pub renderer: Arc<PointCloudRenderer>,
    pub params: PointCloudUniformParams,
    pub rect: egui::Rect,
    /// The plot's Transparency option, resolved for this device.
    pub transparency: Transparency,
}

impl eframe::egui_wgpu::CallbackTrait for PointCloudCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        super::colormap_atlas::prepare(device, queue, callback_resources);
        let mut params = self.params;
        params.screen_aspect = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(queue, &params);
        if self.transparency == Transparency::Oit
            && let Some(atlas) = super::colormap_atlas::bind_group(callback_resources)
        {
            let size = super::oit::frame_size(&self.rect, screen_descriptor.pixels_per_point);
            let renderer = &self.renderer;
            renderer.oit.render(
                device,
                encoder,
                size,
                atlas,
                |variant| renderer.oit_pipeline(device, variant),
                |pass| renderer.draw_points(pass),
            );
        } else if self.transparency != Transparency::Oit {
            self.renderer.oit.release();
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
        let renderer = &self.renderer;
        let pipeline = match self.transparency {
            Transparency::Oit if renderer.oit.paint(rpass) => return,
            Transparency::Oit | Transparency::NoDepthWrite => &renderer.transparent_pipeline,
            Transparency::Off => &renderer.render_pipeline,
        };
        if !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }
        rpass.set_pipeline(pipeline);
        renderer.draw_points(rpass);
    }
}
