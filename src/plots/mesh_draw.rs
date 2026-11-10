//! Mesh draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::channels::ChannelDraw;
use super::common::{Mesh3DUniformParams, MeshVertex3D};
use super::mesh::Mesh3DRenderer;
use super::oit::{Transparency, Variant, build_pipeline};
use std::sync::Arc;

/// The globe and flat heightfield shaders, around `common/mesh3d.wgsl`.
pub(super) const SPHERE_WGSL: &str = crate::assemble_plot_with_coords_shader!(concat!(
    include_str!("shaders/common/mesh3d.wgsl"),
    "\n",
    include_str!("shaders/sphere.wgsl")
));
pub(super) const SURFACE_WGSL: &str = crate::assemble_plot_with_coords_shader!(concat!(
    include_str!("shaders/common/mesh3d.wgsl"),
    "\n",
    include_str!("shaders/surface.wgsl")
));

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

    /// The mesh's pipeline for an OIT [`Variant`] (no culling).
    pub(super) fn oit_pipeline(
        &self,
        device: &wgpu::Device,
        variant: &Variant<'_>,
    ) -> wgpu::RenderPipeline {
        build_pipeline(
            device,
            &self.pipeline_layout,
            &self.shader,
            &[Some(MeshVertex3D::desc())],
            None,
            variant,
        )
    }
}

pub struct Mesh3DCallback {
    pub renderer: Arc<Mesh3DRenderer>,
    pub params: Mesh3DUniformParams,
    pub cube_mode_idx: u32,
    pub rect: egui::Rect,
    /// The plot's Transparency option, resolved for this device.
    pub transparency: Transparency,
    /// A multi-channel composite the GPU blends, baked into the data buffer
    /// (`ChannelRenderer::bake`).
    pub channels: Option<ChannelDraw>,
    /// How the drawn grid thins the planes' cells (`Layer::mesh_thinning`).
    pub thinning: super::Thinning,
}

impl Mesh3DCallback {
    fn cubes(&self) -> bool {
        self.params.mode == self.cube_mode_idx
    }

    /// The pipeline drawing the mesh into egui's pass (or its cached frame).
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        let renderer = &self.renderer;
        match (self.transparency, self.cubes()) {
            (Transparency::Oit | Transparency::NoDepthWrite, _) => &renderer.transparent_pipeline,
            (Transparency::Off, true) => &renderer.voxel_pipeline,
            (Transparency::Off, false) => &renderer.render_pipeline,
        }
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
        self.renderer.update_uniforms(queue, &self.params);
        if let Some(channels) = &self.channels {
            let r = &self.renderer;
            let target = (r.bake_id.id(), &r.data_buffer, self.thinning);
            if channels.bake_into(device, queue, encoder, target) {
                r.inputs.data_changed();
            }
        }
        let size = super::oit::frame_size(&self.rect, screen_descriptor.pixels_per_point);
        let (renderer, cubes) = (&self.renderer, self.cubes());
        let key = renderer.inputs.key((cubes, self.transparency as u8));
        super::frame_cache::render_frame(
            (&renderer.oit, &renderer.frame),
            device,
            encoder,
            (self.transparency, size, key),
            super::colormap_atlas::bind_group(callback_resources),
            (self.pipeline(), |v| renderer.oit_pipeline(device, v)),
            |pass| renderer.draw_geometry(pass, cubes),
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
            |rpass| renderer.draw_geometry(rpass, self.cubes()),
        );
    }
}

pub type SphereRenderer = Mesh3DRenderer;
pub type SphereCallback = Mesh3DCallback;
pub type SurfaceRenderer = Mesh3DRenderer;
pub type SurfaceCallback = Mesh3DCallback;

impl Mesh3DRenderer {
    #[cfg(test)]
    pub fn new_sphere(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        Self::new_sphere_with_coords(
            device,
            target_format,
            matrix_data,
            width,
            height,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_sphere_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        Self::new_with_coords(
            device,
            target_format,
            SPHERE_WGSL,
            Some(wgpu::Face::Back),
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }

    #[cfg(test)]
    pub fn new_surface(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        Self::new_surface_with_coords(
            device,
            target_format,
            matrix_data,
            width,
            height,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_surface_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        Self::new_with_coords(
            device,
            target_format,
            SURFACE_WGSL,
            None,
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }
}
