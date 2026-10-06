//! Mesh pipelines, draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::common::{Mesh3DUniformParams, MeshVertex3D};
use super::mesh::Mesh3DRenderer;
use super::oit::{OitState, Transparency, Variant};
use std::sync::Arc;

/// Mesh pipeline for one [`Variant`] (fragment entry, targets, depth writes).
pub(super) fn mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    variant: &Variant<'_>,
    cull_mode: Option<wgpu::Face>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(variant.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(MeshVertex3D::desc())],
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
            cull_mode,
            front_face: wgpu::FrontFace::Ccw,
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

impl Mesh3DRenderer {
    /// Binds group 0 and the cube or quad template and draws every instance.
    fn draw_geometry(&self, pass: &mut wgpu::RenderPass<'_>, cubes: bool) {
        let (vertices, indices, count) = if cubes {
            (&self.cube_vertex_buffer, &self.cube_index_buffer, 36)
        } else {
            (&self.quad_vertex_buffer, &self.quad_index_buffer, 6)
        };
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..count, 0, 0..self.num_instances);
    }

    /// Draws the mesh into its OIT frame (pipelines built on first use).
    fn render_oit(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: [u32; 2],
        atlas: &wgpu::BindGroup,
        cubes: bool,
    ) {
        let mut oit = self.oit.lock().unwrap_or_else(|p| p.into_inner());
        let state = oit.get_or_insert_with(|| {
            OitState::new(device, self.target_format, |variant| {
                mesh_pipeline(device, &self.pipeline_layout, &self.shader, variant, None)
            })
        });
        state.render(device, encoder, size, atlas, |pass| {
            self.draw_geometry(pass, cubes);
        });
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

pub struct Mesh3DCallback {
    pub renderer: Arc<Mesh3DRenderer>,
    pub params: Mesh3DUniformParams,
    pub cube_mode_idx: u32,
    pub rect: egui::Rect,
    /// The plot's Transparency option, resolved for this device.
    pub transparency: Transparency,
}

impl Mesh3DCallback {
    fn cubes(&self) -> bool {
        self.params.mode == self.cube_mode_idx
    }
}

impl eframe::egui_wgpu::CallbackTrait for Mesh3DCallback {
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
        params.aspect_ratio = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(queue, &params);
        if self.transparency == Transparency::Oit
            && let Some(atlas) = super::colormap_atlas::bind_group(callback_resources)
        {
            let size = super::oit::frame_size(&self.rect, screen_descriptor.pixels_per_point);
            self.renderer
                .render_oit(device, encoder, size, atlas, self.cubes());
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
        let cubes = self.cubes();
        let pipeline = match (self.transparency, cubes) {
            (Transparency::Oit, _) if renderer.paint_oit(rpass) => return,
            (Transparency::Oit | Transparency::NoDepthWrite, _) => &renderer.transparent_pipeline,
            (Transparency::Off, true) => &renderer.voxel_pipeline,
            (Transparency::Off, false) => &renderer.render_pipeline,
        };
        if !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }
        rpass.set_pipeline(pipeline);
        renderer.draw_geometry(rpass, cubes);
    }
}

impl super::common::PlotRenderer for Mesh3DRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

impl super::traits::PlotRenderer for Mesh3DRenderer {
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
        _pointer_pos: egui::Pos2,
        _rect: egui::Rect,
        _data: &crate::data::RenderData,
        _params: &super::traits::PlotRenderParams,
    ) -> Option<super::traits::HoverSample> {
        None
    }
}
