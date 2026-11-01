//! Point cloud draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::channels::ChannelDraw;
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
    /// A multi-channel composite the GPU blends, baked into the data buffer
    /// (`ChannelRenderer::bake`).
    pub channels: Option<ChannelDraw>,
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
        self.renderer.update_uniforms(queue, &self.params);
        if let Some(channels) = &self.channels {
            channels.renderer.update_params(queue, &channels.params);
            let r = &self.renderer;
            let target = (r.bake_id, &r.data_buffer, super::Thinning::NONE);
            if channels
                .renderer
                .bake(device, encoder, &channels.params, target)
            {
                self.renderer.inputs.data_changed();
            }
        }
        let Some(atlas) = super::colormap_atlas::bind_group(callback_resources) else {
            // No frame of this state: `paint` must not draw an older one.
            self.renderer.frame.release();
            self.renderer.oit.release();
            return Vec::new();
        };
        let size = super::oit::frame_size(&self.rect, screen_descriptor.pixels_per_point);
        let renderer = &self.renderer;
        let key = renderer.inputs.key(self.transparency as u8);
        if self.transparency == Transparency::Oit {
            renderer.frame.release();
            renderer.oit.render(
                device,
                encoder,
                (size, key),
                atlas,
                |variant| renderer.oit_pipeline(device, variant),
                |pass| renderer.draw_points(pass),
            );
        } else {
            renderer.oit.release();
            renderer.frame.render(
                device,
                encoder,
                (size, key),
                (self.pipeline(), atlas),
                |pass| renderer.draw_points(pass),
            );
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
        let drawn = match self.transparency {
            Transparency::Oit => renderer.oit.paint(rpass),
            _ => renderer.frame.paint(rpass),
        };
        if drawn || !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }
        // No frame yet (no colormap atlas in `prepare`): drawn directly.
        rpass.set_pipeline(self.pipeline());
        renderer.draw_points(rpass);
    }
}

impl PointCloudCallback {
    /// The pipeline drawing the points into egui's pass (or their cached frame).
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        match self.transparency {
            Transparency::Oit | Transparency::NoDepthWrite => &self.renderer.transparent_pipeline,
            Transparency::Off => &self.renderer.render_pipeline,
        }
    }
}
