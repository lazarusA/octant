use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct OctantVertex {
    pub position: [f32; 3],  // 3D Grid Space (X, Y, Z)
    pub uv: [f32; 2],        // Texture / Data space coordinates
    pub data_value: f32,     // Raw Zarr matrix cell metric scalar
}

impl OctantVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OctantVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct OctantUniforms {
    pub timestep: f32,
    pub colormap: f32,
    pub grid_width: f32,
    pub grid_height: f32,
}

pub struct ShaderRenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_vertices: u32,
}

impl ShaderRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        zarr_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Octant Core Shader Assembly"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders.wgsl"))),
        });

        // 1. Uniform Buffer Setup
        let initial_uniforms = OctantUniforms {
            timestep: 0.0,
            colormap: 0.0,
            grid_width: width as f32,
            grid_height: height as f32,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Octant Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Octant Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Octant Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 2. Build 2D Heatmap Grid Vertices from Zarr Data Array
        let vertices = Self::build_heatmap_vertices(zarr_data, width, height);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Octant Zarr Heatmap Geometry Stream"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Octant Uniform Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Octant Pipeline Frame"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[OctantVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(target_format.into())],
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

        Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            num_vertices: vertices.len() as u32,
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &OctantUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn draw<'pass>(&self, rpass: &mut wgpu::RenderPass<'pass>) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..self.num_vertices, 0..1);
    }

    /// Converts a 2D Zarr matrix into an unstructured grid quad vertex mesh
    fn build_heatmap_vertices(data: &[f32], width: usize, height: usize) -> Vec<OctantVertex> {
        let mut vertices = Vec::with_capacity(width * height * 6);
        let aspect = width as f32 / height as f32;

        let scale_x = 1.8 * (if aspect > 1.0 { 1.0 } else { aspect });
        let scale_y = 1.8 * (if aspect > 1.0 { 1.0 / aspect } else { 1.0 });

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let val = data.get(idx).copied().unwrap_or(0.0);

                let x0 = -0.5 * scale_x + (x as f32 / width as f32) * scale_x;
                let x1 = -0.5 * scale_x + ((x + 1) as f32 / width as f32) * scale_x;

                let y0 = 0.5 * scale_y - (y as f32 / height as f32) * scale_y;
                let y1 = 0.5 * scale_y - ((y + 1) as f32 / height as f32) * scale_y;

                let u0 = x as f32 / width as f32;
                let u1 = (x + 1) as f32 / width as f32;
                let v0 = y as f32 / height as f32;
                let v1 = (y + 1) as f32 / height as f32;

                let v_tl = OctantVertex { position: [x0, y0, 0.0], uv: [u0, v0], data_value: val };
                let v_tr = OctantVertex { position: [x1, y0, 0.0], uv: [u1, v0], data_value: val };
                let v_bl = OctantVertex { position: [x0, y1, 0.0], uv: [u0, v1], data_value: val };
                let v_br = OctantVertex { position: [x1, y1, 0.0], uv: [u1, v1], data_value: val };

                // Triangle 1
                vertices.push(v_tl);
                vertices.push(v_bl);
                vertices.push(v_tr);

                // Triangle 2
                vertices.push(v_tr);
                vertices.push(v_bl);
                vertices.push(v_br);
            }
        }

        vertices
    }
}

/// Custom egui_wgpu render callback that updates GPU uniforms in prepare() and executes draw calls in paint()
pub struct OctantRenderCallback {
    pub pipeline: Arc<ShaderRenderPipeline>,
    pub uniforms: OctantUniforms,
}

impl eframe::egui_wgpu::CallbackTrait for OctantRenderCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.pipeline.update_uniforms(queue, &self.uniforms);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let viewport = info.clip_rect_in_pixels();
        if viewport.width_px == 0 || viewport.height_px == 0 {
            return;
        }

        let left = viewport.left_px.max(0) as u32;
        let top = viewport.top_px.max(0) as u32;
        let width = viewport.width_px.max(0) as u32;
        let height = viewport.height_px.max(0) as u32;

        rpass.set_viewport(
            viewport.left_px as f32,
            viewport.top_px as f32,
            viewport.width_px as f32,
            viewport.height_px as f32,
            0.0,
            1.0,
        );

        rpass.set_scissor_rect(left, top, width, height);

        self.pipeline.draw(rpass);
    }
}
