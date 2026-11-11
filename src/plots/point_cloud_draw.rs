//! Point cloud draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::channels::ChannelDraw;
use super::oit::{Transparency, Variant, build_pipeline};
use super::point_cloud::{
    PointCloudRenderer, PointCloudUniformParams, PointCloudUniforms, PointCloudVertex,
};
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

impl PointCloudRenderer {
    /// Writes the uniforms of a frame drawn with `params`, recording them
    /// for the frame cache.
    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &PointCloudUniformParams) {
        let instance_cnt = self.instance_count.load(Ordering::Relaxed);
        let depth =
            super::common::calculate_3d_depth(instance_cnt as usize, params.width, params.height);
        let uniforms = PointCloudUniforms {
            camera: params.camera,
            slice: params.slice,
            aspect_x: params.aspect_x,
            aspect_y: params.aspect_y,
            aspect_z: params.aspect_z,
            point_size: params.point_size,
            width: params.width.max(1),
            height: params.height.max(1),
            depth,
            shift_x: params.shift_x,
            shift_y: params.shift_y,
            shift_z: params.shift_z,
            scale_by_value: u32::from(params.scale_by_value),
            _pad1: 0,
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        self.inputs.uniforms_written(bytemuck::bytes_of(&uniforms));
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
            let r = &self.renderer;
            let target = (r.bake_id.id(), &r.data_buffer, r.thinning);
            if channels.bake_into(device, queue, encoder, target) {
                r.inputs.data_changed();
            }
        }
        let size = super::oit::frame_size(&self.rect, screen_descriptor.pixels_per_point);
        let renderer = &self.renderer;
        let key = renderer.inputs.key(self.transparency as u8);
        super::frame_cache::render_frame(
            (&renderer.oit, &renderer.frame),
            device,
            encoder,
            (self.transparency, size, key),
            (super::colormap_atlas::bind_group(callback_resources), None),
            (self.pipeline(), |v| renderer.oit_pipeline(device, v)),
            |pass| renderer.draw_points(pass),
        );
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
        super::frame_cache::paint_frame(
            (&renderer.oit, &renderer.frame),
            (self.transparency, self.pipeline()),
            rpass,
            callback_resources,
            |rpass| renderer.draw_points(rpass),
        );
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
