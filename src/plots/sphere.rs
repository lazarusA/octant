use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl SphereVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SphereVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub sphere_mode: u32,
    pub width: u32,
    pub height: u32,
    pub color: super::common::PlotColorParams,
}

pub struct SphereRenderer {
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

impl SphereRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/sphere.wgsl"));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sphere Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let num_instances = (width * height) as u32;

        let initial_uniforms = SphereUniforms {
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_ratio: 1.0,
            zoom: 2.5,
            displacement_strength: 0.5,
            sphere_mode: 0,
            width: width as u32,
            height: height as u32,
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Sphere Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer =
            super::common::create_storage_buffer(device, "Sphere Data Storage Buffer", matrix_data);

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Sphere Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Sphere Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sphere Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sphere Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(SphereVertex::desc())],
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

        // Dedicated pipeline for Mode 3 Radial Lego Cubes with hardware backface culling
        // (same cull_mode as the default pipeline — globe modes already benefit from Back culling;
        //  this separate binding lets us keep cube draw calls clearly separated in paint())
        let voxel_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sphere Voxel Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(SphereVertex::desc())],
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

        // 1. Instanced Unit Quad Template for Smooth Globe, Smooth Terrain, and Flat Steps
        let (quad_vertices, quad_indices) = Self::build_unit_quad();
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Unit Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Unit Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // 2. Unit Cube Template for GPU Instanced 3D Radial Lego Cubes
        let (cube_vertices, cube_indices) = Self::build_unit_cube();
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Unit Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Unit Cube Index Buffer"),
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

    #[allow(clippy::too_many_arguments)]
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        color: &super::common::PlotColorParams,
        rotation_y: f32,
        rotation_x: f32,
        aspect_ratio: f32,
        zoom: f32,
        displacement_strength: f32,
        sphere_mode: u32,
    ) {
        let uniforms = SphereUniforms {
            rotation_y,
            rotation_x,
            aspect_ratio: aspect_ratio.max(0.1),
            zoom,
            displacement_strength,
            sphere_mode,
            width: self.width as u32,
            height: self.height as u32,
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "SphereRenderer::update_data",
        );
    }

    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'_>, mode: u32) {
        let is_voxel = mode == 3;
        let pipeline = if is_voxel {
            &self.voxel_pipeline
        } else {
            &self.render_pipeline
        };
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        if is_voxel {
            rpass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
            rpass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..36, 0, 0..self.num_instances);
        } else {
            rpass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            rpass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..6, 0, 0..self.num_instances);
        }
    }

    fn build_unit_quad() -> (Vec<SphereVertex>, Vec<u32>) {
        super::common::build_unit_quad_mesh(|position, uv, normal| SphereVertex {
            position,
            uv,
            normal,
        })
    }

    fn build_unit_cube() -> (Vec<SphereVertex>, Vec<u32>) {
        super::common::build_unit_cube_mesh(|position, uv, normal| SphereVertex {
            position,
            uv,
            normal,
        })
    }
}

impl super::common::PlotRenderer for SphereRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

pub struct SphereCallback {
    pub renderer: Arc<SphereRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub sphere_mode: u32,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for SphereCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let aspect_ratio = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.rotation_y,
            self.rotation_x,
            aspect_ratio,
            self.zoom,
            self.displacement_strength,
            self.sphere_mode,
        );
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

        if self.sphere_mode == 3 {
            // Mode 3: 3D Radial Lego Cubes (Dedicated pipeline with hardware backface culling)
            rpass.set_pipeline(&self.renderer.voxel_pipeline);
            rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.renderer.cube_vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.renderer.cube_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..36, 0, 0..self.renderer.num_instances);
        } else {
            // Mode 0 (Smooth Globe), Mode 1 (Smooth Terrain), & Mode 2 (Flat Steps) - GPU Instanced Unit Quad Draw
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

pub struct SpherePlot;
