use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub cell_index: u32,
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
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereUniforms {
    pub colormap: u32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

pub struct SphereRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
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
            label: Some("3D Sphere Globe Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let initial_uniforms = SphereUniforms {
            colormap: 0,
            rotation_y: 0.0,
            rotation_x: 0.2,
            aspect_ratio: 1.0,
            zoom: 2.5,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // GPU Storage Buffer for 1D scalar data channel (4.1 MB for 720x1440 resolution)
        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Data Storage Buffer"),
            contents: bytemuck::cast_slice(matrix_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sphere Bind Group Layout"),
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
            label: Some("Sphere Bind Group"),
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
            label: Some("Sphere Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sphere Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SphereVertex::desc()],
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
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (vertices, indices) = Self::build_sphere_mesh(width, height);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
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
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, colormap: u32, rotation_y: f32, rotation_x: f32, aspect_ratio: f32, zoom: f32) {
        let uniforms = SphereUniforms {
            colormap,
            rotation_y,
            rotation_x,
            aspect_ratio: aspect_ratio.max(0.1),
            zoom,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Fast GPU Storage Buffer data channel upload (4.1 MB per frame, zero vertex re-allocation)
    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    /// Build static 1:1 indexed 3D sphere mesh geometry.
    fn build_sphere_mesh(width: usize, height: usize) -> (Vec<SphereVertex>, Vec<u32>) {
        let rings = height;
        let sectors = width;
        let radius = 1.0f32;

        let num_quads = rings * sectors;
        let mut vertices = Vec::with_capacity(num_quads * 4);
        let mut indices = Vec::with_capacity(num_quads * 6);

        for r in 0..rings {
            for s in 0..sectors {
                let cell_index = (r * width + s) as u32;

                let u0 = s as f32 / sectors as f32;
                let u1 = (s + 1) as f32 / sectors as f32;
                let v0 = r as f32 / rings as f32;
                let v1 = (r + 1) as f32 / rings as f32;

                let p_tl = Self::spherical_to_cartesian(radius, u0, v0);
                let p_tr = Self::spherical_to_cartesian(radius, u1, v0);
                let p_bl = Self::spherical_to_cartesian(radius, u0, v1);
                let p_br = Self::spherical_to_cartesian(radius, u1, v1);

                let base_idx = vertices.len() as u32;

                vertices.push(SphereVertex { position: p_tl, uv: [u0, v0], cell_index });
                vertices.push(SphereVertex { position: p_tr, uv: [u1, v0], cell_index });
                vertices.push(SphereVertex { position: p_bl, uv: [u0, v1], cell_index });
                vertices.push(SphereVertex { position: p_br, uv: [u1, v1], cell_index });

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

    /// Map UV coordinates (u in [0..1], v in [0..1]) to 3D Cartesian coordinates [X, Y, Z] on sphere
    fn spherical_to_cartesian(radius: f32, u: f32, v: f32) -> [f32; 3] {
        let lon = (u - 0.5) * 2.0 * std::f32::consts::PI;
        let lat = (0.5 - v) * std::f32::consts::PI;

        let cos_lat = lat.cos();
        let sin_lat = lat.sin();

        let x = radius * cos_lat * lon.sin();
        let y = radius * sin_lat;
        let z = radius * cos_lat * lon.cos();

        [x, y, z]
    }
}

pub struct SphereCallback {
    pub renderer: Arc<SphereRenderer>,
    pub colormap: u32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub zoom: f32,
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
        let aspect_ratio = self.rect.width() / self.rect.height().max(1.0);
        self.renderer.update_uniforms(queue, self.colormap, self.rotation_y, self.rotation_x, aspect_ratio, self.zoom);
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

pub struct SpherePlot;
