//! Point cloud pipelines, draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::oit::{OitState, Transparency, Variant};
use super::point_cloud::{PointCloudRenderer, PointCloudUniformParams, PointCloudVertex};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Point billboard pipeline for one [`Variant`] (fragment entry, targets,
/// depth writes).
pub(super) fn point_cloud_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    variant: &Variant<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(variant.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(PointCloudVertex::desc())],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(variant.entry),
            targets: variant.targets,
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(super::common::default_depth_stencil_state(
            variant.depth_write,
            wgpu::CompareFunction::LessEqual,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

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

    /// Draws the points into their OIT frame (pipelines built on first use).
    fn render_oit(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: [u32; 2],
        atlas: &wgpu::BindGroup,
    ) {
        let mut oit = self.oit.lock().unwrap_or_else(|p| p.into_inner());
        let state = oit.get_or_insert_with(|| {
            OitState::new(device, self.target_format, |variant| {
                point_cloud_pipeline(device, &self.pipeline_layout, &self.shader, variant)
            })
        });
        state.render(device, encoder, size, atlas, |pass| self.draw_points(pass));
    }

    /// Frees the OIT frame when this renderer draws without OIT.
    pub(crate) fn release_oit_frame(&self) {
        let mut oit = self.oit.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(state) = oit.as_mut() {
            state.release_frame();
        }
    }

    fn paint_oit(&self, rpass: &mut wgpu::RenderPass<'static>) -> bool {
        let oit = self.oit.lock().unwrap_or_else(|p| p.into_inner());
        oit.as_ref().is_some_and(|state| state.paint(rpass))
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
            self.renderer.render_oit(device, encoder, size, atlas);
        } else if self.transparency != Transparency::Oit {
            self.renderer.release_oit_frame();
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
            Transparency::Oit if renderer.paint_oit(rpass) => return,
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
