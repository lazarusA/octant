use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::common::{Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D};

pub struct SurfaceRenderer {
    render_pipeline: wgpu::RenderPipeline,
    voxel_pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_instances: u32,
    width: usize,
    height: usize,
}

impl SurfaceRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/surface.wgsl"));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Surface / Blocks Shader Module"),
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
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Surface Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Surface Data Storage Buffer",
            matrix_data,
        );

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Surface Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Surface Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surface Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surface Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(MeshVertex3D::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None, // Two-sided rendering for open heightfield surfaces
                ..Default::default()
            },
            depth_stencil: Some(super::common::default_depth_stencil_state(
                true,
                wgpu::CompareFunction::LessEqual,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Dedicated pipeline for Mode 2 Lego Cubes with hardware backface culling
        // (eliminates internal hidden voxel cube faces)
        let voxel_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surface Voxel Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(MeshVertex3D::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(super::common::default_depth_stencil_state(
                true,
                wgpu::CompareFunction::LessEqual,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 1. Instanced Unit Quad Template for Smooth Terrain & Flat Steps
        let (quad_vertices, quad_indices) = Self::build_unit_quad();
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Unit Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Unit Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // 2. Unit Cube Template for GPU Instanced 3D Lego Cubes
        let (cube_vertices, cube_indices) = Self::build_unit_cube();
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Unit Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Unit Cube Index Buffer"),
            contents: bytemuck::cast_slice(&cube_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            voxel_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            cube_vertex_buffer,
            cube_index_buffer,
            data_buffer,
            uniform_buffer,
            bind_group,
            num_instances,
            width,
            height,
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
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "SurfaceRenderer::update_data",
        );
    }

    fn build_unit_quad() -> (Vec<MeshVertex3D>, Vec<u32>) {
        super::common::build_unit_quad_mesh(|position, uv, normal| MeshVertex3D {
            position,
            uv,
            normal,
        })
    }

    fn build_unit_cube() -> (Vec<MeshVertex3D>, Vec<u32>) {
        super::common::build_unit_cube_mesh(|position, uv, normal| MeshVertex3D {
            position,
            uv,
            normal,
        })
    }
}

impl super::common::PlotRenderer for SurfaceRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

pub struct SurfaceCallback {
    pub renderer: Arc<SurfaceRenderer>,
    pub params: Mesh3DUniformParams,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for SurfaceCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut params = self.params.clone();
        params.aspect_ratio = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(queue, &params);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if !super::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }

        if self.params.mode == 2 {
            // Mode 2: 3D Lego Cubes (Dedicated pipeline with hardware backface culling)
            rpass.set_pipeline(&self.renderer.voxel_pipeline);
            rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.renderer.cube_vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.renderer.cube_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..36, 0, 0..self.renderer.num_instances);
        } else {
            // Mode 0 (Smooth Terrain) & Mode 1 (Flat Steps) - Two-Sided GPU Instanced Unit Quad Draw
            rpass.set_pipeline(&self.renderer.render_pipeline);
            rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.renderer.quad_vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.renderer.quad_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..6, 0, 0..self.renderer.num_instances);
        }
    }
}

pub struct SurfacePlot;
