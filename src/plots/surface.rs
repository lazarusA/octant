use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
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
    pub colormap: u32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub surface_mode: u32,
    pub width: u32,
}

pub struct SurfaceRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
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
            colormap: 0,
            rotation_y: 0.0,
            rotation_x: 0.4,
            aspect_ratio: 1.0,
            zoom: 2.5,
            displacement_strength: 1.0,
            surface_mode: 0,
            width: width as u32,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Data Storage Buffer"),
            contents: bytemuck::cast_slice(matrix_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surface Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Surface Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: data_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surface Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surface Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SurfaceVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (vertices, indices) = Self::build_mesh(width, height);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surface Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            data_buffer,
            uniform_buffer,
            bind_group,
            num_indices: indices.len() as u32,
            width,
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        colormap: u32,
        rotation_y: f32,
        rotation_x: f32,
        aspect_ratio: f32,
        zoom: f32,
        displacement_strength: f32,
        surface_mode: u32,
    ) {
        let uniforms = SurfaceUniforms {
            colormap,
            rotation_y,
            rotation_x,
            aspect_ratio: aspect_ratio.max(0.1),
            zoom,
            displacement_strength,
            surface_mode,
            width: self.width as u32,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    fn build_mesh(width: usize, height: usize) -> (Vec<SurfaceVertex>, Vec<u32>) {
        let num_quads = width * height;
        let mut vertices = Vec::with_capacity(num_quads * 4);
        let mut indices = Vec::with_capacity(num_quads * 6);

        // Dynamic data aspect ratio scaling matching tensor shape
        let data_aspect = (width as f32 / height as f32).max(0.1);
        let scale_x = 2.0 * data_aspect;
        let scale_y = 2.0;

        let norm_up = [0.0, 1.0, 0.0];

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

                // Top Face (position.z = 1.0 = top height)
                vertices.push(SurfaceVertex { position: [x0, y0, 1.0], uv: [u0, v0], cell_index, corner_index: corner_tl, normal: norm_up });
                vertices.push(SurfaceVertex { position: [x1, y0, 1.0], uv: [u1, v0], cell_index, corner_index: corner_tr, normal: norm_up });
                vertices.push(SurfaceVertex { position: [x0, y1, 1.0], uv: [u0, v1], cell_index, corner_index: corner_bl, normal: norm_up });
                vertices.push(SurfaceVertex { position: [x1, y1, 1.0], uv: [u1, v1], cell_index, corner_index: corner_br, normal: norm_up });

                // Triangle 1: TL, BL, TR
                indices.push(base_idx);
                indices.push(base_idx + 2);
                indices.push(base_idx + 1);

                // Triangle 2: TR, BL, BR
                indices.push(base_idx + 1);
                indices.push(base_idx + 2);
                indices.push(base_idx + 3);
            }
        }

        (vertices, indices)
    }
}

pub struct SurfaceCallback {
    pub renderer: Arc<SurfaceRenderer>,
    pub colormap: u32,
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
        let aspect_ratio = self.rect.width() / self.rect.height().max(1.0);
        self.renderer.update_uniforms(
            queue,
            self.colormap,
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
        let ppp = info.pixels_per_point;
        let px_x = (self.rect.min.x * ppp).max(0.0) as u32;
        let px_y = (self.rect.min.y * ppp).max(0.0) as u32;
        let px_w = (self.rect.width() * ppp).max(1.0) as u32;
        let px_h = (self.rect.height() * ppp).max(1.0) as u32;

        rpass.set_viewport(px_x as f32, px_y as f32, px_w as f32, px_h as f32, 0.0, 1.0);
        rpass.set_scissor_rect(px_x, px_y, px_w, px_h);

        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.renderer.num_indices, 0, 0..1);
    }
}

pub struct SurfacePlot;
