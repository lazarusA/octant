use wgpu::util::DeviceExt;

use super::common::{Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D};
pub use super::mesh_draw::Mesh3DCallback;
use super::oit::{OitSlot, Variant, build_pipeline, egui_target};

pub struct Mesh3DRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub voxel_pipeline: wgpu::RenderPipeline,
    /// Translucent quads (cull_mode, no depth writes).
    pub transparent_pipeline: wgpu::RenderPipeline,
    /// Translucent voxel cubes (backface culling, no depth writes).
    pub transparent_voxel_pipeline: wgpu::RenderPipeline,
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub cube_vertex_buffer: wgpu::Buffer,
    pub cube_index_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    /// Names `data_buffer` to the channel bake.
    pub bake_id: super::BakeSlot,
    pub coord_x_buffer: wgpu::Buffer,
    pub coord_y_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    /// Where the heights come from (bind group 2).
    pub(crate) heights: super::mesh_heights::HeightSlot,
    pub num_instances: u32,
    pub width: usize,
    pub height: usize,
    /// Kept to build the OIT pipelines on first use.
    pub(super) shader: wgpu::ShaderModule,
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) oit: OitSlot,
    /// Its frame when drawn without OIT.
    pub(crate) frame: super::frame_cache::FrameSlot,
    /// What its OIT frame depends on.
    pub(crate) inputs: super::oit::FrameInputs,
    pub(crate) cull_mode: Option<wgpu::Face>,
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
            camera: super::camera::OrbitCamera::default().uniform(),
            slice: super::slice::SliceUniform::OFF,
            displacement_strength: 0.5,
            mode: 0,
            width: width as u32,
            height: height as u32,
            coord_mode: 0,
            height_flags: 0,
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            height_range: [0.0, 1.0],
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

        let heights = super::mesh_heights::HeightSlot::new(device, &data_buffer);
        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh 3D Pipeline Layout"),
            bind_group_layouts: &[
                Some(&bind_group_layout),
                Some(&colormap_layout),
                Some(heights.layout()),
            ],
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
        let transparent_pipeline =
            pipeline("Mesh 3D Transparent Render Pipeline", cull_mode, false);
        let transparent_voxel_pipeline = pipeline(
            "Mesh 3D Transparent Voxel Render Pipeline",
            Some(wgpu::Face::Back),
            false,
        );

        let (quad_vertex_buffer, quad_index_buffer, cube_vertex_buffer, cube_index_buffer) =
            create_template_buffers(device);

        Self {
            render_pipeline,
            voxel_pipeline,
            transparent_pipeline,
            transparent_voxel_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            cube_vertex_buffer,
            cube_index_buffer,
            data_buffer,
            bake_id: super::BakeSlot::default(),
            coord_x_buffer,
            coord_y_buffer,
            uniform_buffer,
            bind_group,
            heights,
            num_instances,
            width,
            height,
            shader,
            pipeline_layout,
            oit: OitSlot::new(target_format),
            frame: super::frame_cache::FrameSlot::new(target_format),
            inputs: Default::default(),
            cull_mode,
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &Mesh3DUniformParams) {
        let uniforms = Mesh3DUniforms {
            camera: params.camera,
            slice: params.slice,
            displacement_strength: params.displacement_strength,
            mode: params.mode,
            width: self.width as u32,
            height: self.height as u32,
            coord_mode: params.coord_mode,
            height_flags: u32::from(params.flip_height) | (u32::from(params.height_composite) << 1),
            lon_bounds: params.lon_bounds,
            lat_bounds: params.lat_bounds,
            height_range: params.height_range,
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        self.inputs.uniforms_written(bytemuck::bytes_of(&uniforms));
    }

    pub fn update_coords(&self, queue: &wgpu::Queue, coords_x: &[f32], coords_y: &[f32]) {
        self.inputs.data_changed();
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
        self.inputs.data_changed();
        self.bake_id.renew();
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "Mesh3DRenderer::update_data",
        );
    }
}

pub(crate) fn pad_coord_buffer(coord: Option<&[f32]>, min_capacity: usize) -> Vec<f32> {
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
        super::common::build_unit_quad_mesh(|position, normal| MeshVertex3D { position, normal });
    let (cube_verts, cube_idx) =
        super::common::build_unit_cube_mesh(|position, normal| MeshVertex3D { position, normal });
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
