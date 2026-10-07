use wgpu::util::DeviceExt;

use super::common::{Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D};
pub use super::mesh_draw::Mesh3DCallback;
use super::oit::{OitSlot, Variant, build_pipeline, egui_target};

pub struct Mesh3DRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub voxel_pipeline: wgpu::RenderPipeline,
    /// Both modes with translucent colors: no culling, no depth writes.
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub cube_vertex_buffer: wgpu::Buffer,
    pub cube_index_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    pub coord_x_buffer: wgpu::Buffer,
    pub coord_y_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub num_instances: u32,
    pub width: usize,
    pub height: usize,
    /// Kept to build the OIT pipelines on first use.
    pub(super) shader: wgpu::ShaderModule,
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) oit: OitSlot,
}

impl Mesh3DRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        shader_source: &str,
        cull_mode: Option<wgpu::Face>,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        Self::new_with_coords(
            device,
            target_format,
            shader_source,
            cull_mode,
            matrix_data,
            width,
            height,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        shader_source: &str,
        cull_mode: Option<wgpu::Face>,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh 3D Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let num_instances = (width * height) as u32;

        let initial_uniforms = Mesh3DUniforms {
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_ratio: 1.0,
            zoom: 2.5,
            displacement_strength: 0.5,
            mode: 0,
            width: width as u32,
            height: height as u32,
            coord_mode: 0,
            has_reference_globe: 0,
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            _pad: [0; 2],
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Mesh 3D Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Mesh 3D Data Storage Buffer",
            matrix_data,
        );

        let padded_coords_x = pad_coord_buffer(coord_x, width.max(128));
        let padded_coords_y = pad_coord_buffer(coord_y, height.max(128));

        let coord_x_buffer = super::common::create_storage_buffer(
            device,
            "Mesh 3D Coord X Storage Buffer",
            &padded_coords_x,
        );

        let coord_y_buffer = super::common::create_storage_buffer(
            device,
            "Mesh 3D Coord Y Storage Buffer",
            &padded_coords_y,
        );

        let bind_group_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Mesh 3D Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::VERTEX,
        );

        let bind_group = super::common::create_plot_with_coords_bind_group(
            device,
            "Mesh 3D Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
            &coord_x_buffer,
            &coord_y_buffer,
        );

        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh 3D Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&colormap_layout)],
            immediate_size: 0,
        });

        let target = egui_target(target_format);
        let pipeline = |label, cull_mode, depth_write| {
            let variant = Variant::egui(label, &target, depth_write);
            build_pipeline(
                device,
                &pipeline_layout,
                &shader,
                &[Some(MeshVertex3D::desc())],
                cull_mode,
                &variant,
            )
        };
        let render_pipeline = pipeline("Mesh 3D Render Pipeline", cull_mode, true);
        let voxel_pipeline = pipeline(
            "Mesh 3D Voxel Render Pipeline",
            Some(wgpu::Face::Back),
            true,
        );
        let transparent_pipeline = pipeline("Mesh 3D Transparent Render Pipeline", None, false);

        let (quad_vertex_buffer, quad_index_buffer, cube_vertex_buffer, cube_index_buffer) =
            create_template_buffers(device);

        Self {
            render_pipeline,
            voxel_pipeline,
            transparent_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            cube_vertex_buffer,
            cube_index_buffer,
            data_buffer,
            coord_x_buffer,
            coord_y_buffer,
            uniform_buffer,
            bind_group,
            num_instances,
            width,
            height,
            shader,
            pipeline_layout,
            oit: OitSlot::new(target_format),
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &Mesh3DUniformParams) {
        let uniforms = Mesh3DUniforms {
            rotation_y: params.rotation_y,
            rotation_x: params.rotation_x,
            aspect_ratio: params.aspect_ratio.max(0.1),
            zoom: params.zoom,
            displacement_strength: params.displacement_strength,
            mode: params.mode,
            width: self.width as u32,
            height: self.height as u32,
            coord_mode: params.coord_mode,
            has_reference_globe: if params.has_reference_globe { 1 } else { 0 },
            lon_bounds: params.lon_bounds,
            lat_bounds: params.lat_bounds,
            _pad: [0; 2],
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_coords(&self, queue: &wgpu::Queue, coords_x: &[f32], coords_y: &[f32]) {
        if !coords_x.is_empty() {
            super::common::safe_write_buffer(
                queue,
                &self.coord_x_buffer,
                coords_x,
                "Mesh3DRenderer::update_coords_x",
            );
        }
        if !coords_y.is_empty() {
            super::common::safe_write_buffer(
                queue,
                &self.coord_y_buffer,
                coords_y,
                "Mesh3DRenderer::update_coords_y",
            );
        }
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "Mesh3DRenderer::update_data",
        );
    }
}

fn pad_coord_buffer(coord: Option<&[f32]>, min_capacity: usize) -> Vec<f32> {
    let dummy = [0.0f32; 4];
    let mut padded = coord.filter(|s| !s.is_empty()).unwrap_or(&dummy).to_vec();
    if padded.len() < min_capacity {
        padded.resize(min_capacity, 0.0);
    }
    padded
}

fn create_template_buffers(
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let (quad_verts, quad_idx) =
        super::common::build_unit_quad_mesh(|position, uv, normal| MeshVertex3D {
            position,
            uv,
            normal,
        });
    let (cube_verts, cube_idx) =
        super::common::build_unit_cube_mesh(|position, uv, normal| MeshVertex3D {
            position,
            uv,
            normal,
        });
    let buf = |label, contents, usage| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage,
        })
    };
    (
        buf(
            "Mesh 3D Quad VBO",
            bytemuck::cast_slice(&quad_verts),
            wgpu::BufferUsages::VERTEX,
        ),
        buf(
            "Mesh 3D Quad IBO",
            bytemuck::cast_slice(&quad_idx),
            wgpu::BufferUsages::INDEX,
        ),
        buf(
            "Mesh 3D Cube VBO",
            bytemuck::cast_slice(&cube_verts),
            wgpu::BufferUsages::VERTEX,
        ),
        buf(
            "Mesh 3D Cube IBO",
            bytemuck::cast_slice(&cube_idx),
            wgpu::BufferUsages::INDEX,
        ),
    )
}
