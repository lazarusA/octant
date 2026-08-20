use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SurfaceVertex {
    pub position: [f32; 3], // x, y, base_y_factor (1.0 = top face, 0.0 = base floor)
    pub uv: [f32; 2],
    pub cell_index: u32,
    pub corner_index: u32,
    pub normal: [f32; 3],
}

impl SurfaceVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SurfaceVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SurfaceUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub surface_mode: u32,
    pub width: u32,
    pub _pad0: u32,
    pub color: super::common::PlotColorParams,
}

pub struct SurfaceRenderer {
    render_pipeline: wgpu::RenderPipeline,
    grid_vertex_buffer: wgpu::Buffer,
    grid_index_buffer: wgpu::Buffer,
    grid_num_indices: u32,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_instances: u32,
    width: usize,
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

        let initial_uniforms = SurfaceUniforms {
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_ratio: 1.0,
            zoom: 2.5,
            displacement_strength: 0.5,
            surface_mode: 0,
            width: width as u32,
            _pad0: 0,
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
                buffers: &[Some(SurfaceVertex::desc())],
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
                cull_mode: None,
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

        // 1. Static Grid Mesh for Smooth Terrain & Flat Steps
        let (grid_vertices, grid_indices) = Self::build_grid_mesh(width, height);
        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&grid_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let grid_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Grid Index Buffer"),
            contents: bytemuck::cast_slice(&grid_indices),
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
            grid_vertex_buffer,
            grid_index_buffer,
            grid_num_indices: grid_indices.len() as u32,
            cube_vertex_buffer,
            cube_index_buffer,
            data_buffer,
            uniform_buffer,
            bind_group,
            num_instances: (width * height) as u32,
            width,
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
        surface_mode: u32,
    ) {
        let uniforms = SurfaceUniforms {
            rotation_y,
            rotation_x,
            aspect_ratio: aspect_ratio.max(0.1),
            zoom,
            displacement_strength,
            surface_mode,
            width: self.width as u32,
            _pad0: 0,
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    fn build_grid_mesh(width: usize, height: usize) -> (Vec<SurfaceVertex>, Vec<u32>) {
        let num_quads = width * height;
        let mut vertices = Vec::with_capacity(num_quads * 4);
        let mut indices = Vec::with_capacity(num_quads * 6);

        let data_aspect = (width as f32 / height as f32).max(0.1);
        let scale_x = 2.0 * data_aspect;
        let scale_y = 2.0;

        let norm_top = [0.0, 1.0, 0.0];

        for y in 0..height {
            for x in 0..width {
                let cell_index = (y * width + x) as u32;

                let x0 = -data_aspect + (x as f32 / width as f32) * scale_x;
                let x1 = -data_aspect + ((x + 1) as f32 / width as f32) * scale_x;

                let y0 = -1.0 + (y as f32 / height as f32) * scale_y;
                let y1 = -1.0 + ((y + 1) as f32 / height as f32) * scale_y;

                let u0 = x as f32 / width as f32;
                let u1 = (x + 1) as f32 / width as f32;
                let v0 = y as f32 / height as f32;
                let v1 = (y + 1) as f32 / height as f32;

                let corner_tl = (y * (width + 1) + x) as u32;
                let corner_tr = (y * (width + 1) + (x + 1)) as u32;
                let corner_bl = ((y + 1) * (width + 1) + x) as u32;
                let corner_br = ((y + 1) * (width + 1) + (x + 1)) as u32;

                let base_idx = vertices.len() as u32;

                vertices.push(SurfaceVertex {
                    position: [x0, y0, 1.0],
                    uv: [u0, v0],
                    cell_index,
                    corner_index: corner_tl,
                    normal: norm_top,
                });
                vertices.push(SurfaceVertex {
                    position: [x1, y0, 1.0],
                    uv: [u1, v0],
                    cell_index,
                    corner_index: corner_tr,
                    normal: norm_top,
                });
                vertices.push(SurfaceVertex {
                    position: [x0, y1, 1.0],
                    uv: [u0, v1],
                    cell_index,
                    corner_index: corner_bl,
                    normal: norm_top,
                });
                vertices.push(SurfaceVertex {
                    position: [x1, y1, 1.0],
                    uv: [u1, v1],
                    cell_index,
                    corner_index: corner_br,
                    normal: norm_top,
                });

                indices.push(base_idx);
                indices.push(base_idx + 2);
                indices.push(base_idx + 1);

                indices.push(base_idx + 1);
                indices.push(base_idx + 2);
                indices.push(base_idx + 3);
            }
        }

        (vertices, indices)
    }

    fn build_unit_cube() -> (Vec<SurfaceVertex>, Vec<u32>) {
        super::common::build_unit_cube_mesh(|position, uv, normal| SurfaceVertex {
            position,
            uv,
            cell_index: 0,
            corner_index: 0,
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
    pub color_params: super::common::PlotColorParams,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub surface_mode: u32,
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
        let aspect_ratio = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.rotation_y,
            self.rotation_x,
            aspect_ratio,
            self.zoom,
            self.displacement_strength,
            self.surface_mode,
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

        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);

        if self.surface_mode == 2 {
            // Mode 2: 3D Lego Cubes (GPU Instanced Unit Cube Draw)
            rpass.set_vertex_buffer(0, self.renderer.cube_vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.renderer.cube_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..36, 0, 0..self.renderer.num_instances);
        } else {
            // Mode 0 (Smooth Terrain) & Mode 1 (Flat Steps)
            rpass.set_vertex_buffer(0, self.renderer.grid_vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.renderer.grid_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..self.renderer.grid_num_indices, 0, 0..1);
        }
    }
}

pub struct SurfacePlot;
