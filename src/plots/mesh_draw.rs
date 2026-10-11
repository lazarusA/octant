//! Mesh draw calls and the egui paint callback, with the
//! transparency modes of [`Transparency`].

use super::common::{Mesh3DUniformParams, MeshVertex3D};
use super::mesh::Mesh3DRenderer;
use super::oit::{Transparency, Variant, build_pipeline};
use std::sync::Arc;

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
            let (renderer, cubes) = (&self.renderer, self.cubes());
            renderer.oit.render(
                device,
                encoder,
                size,
                atlas,
                |variant| renderer.oit_pipeline(device, variant),
                |pass| renderer.draw_geometry(pass, cubes),
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
        let cubes = self.cubes();
        let pipeline = match (self.transparency, cubes) {
            (Transparency::Oit, _) if renderer.oit.paint(rpass) => return,
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

pub type SphereRenderer = Mesh3DRenderer;
pub type SphereCallback = Mesh3DCallback;
pub type SurfaceRenderer = Mesh3DRenderer;
pub type SurfaceCallback = Mesh3DCallback;

impl Mesh3DRenderer {
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
        let shader_source =
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/sphere.wgsl"));
        Self::new_with_coords(
            device,
            target_format,
            shader_source,
            Some(wgpu::Face::Back),
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }

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
        let shader_source =
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/surface.wgsl"));
        Self::new_with_coords(
            device,
            target_format,
            shader_source,
            None,
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }
}
