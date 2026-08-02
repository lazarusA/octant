use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub val: f32,
}

impl HeatmapVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<HeatmapVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapUniforms {
    pub colormap: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

pub struct HeatmapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_vertices: u32,
}

impl HeatmapRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/heatmap.wgsl"));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Heatmap Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let initial_uniforms = HeatmapUniforms {
            colormap: 0,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Heatmap Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Heatmap Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Heatmap Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Heatmap Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Heatmap Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[HeatmapVertex::desc()],
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

        let vertices = Self::build_mesh(matrix_data, width, height);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Heatmap Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            num_vertices: vertices.len() as u32,
        }
    }

    pub fn update_colormap(&self, queue: &wgpu::Queue, colormap: u32) {
        let uniforms = HeatmapUniforms {
            colormap,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn build_mesh(data: &[f32], width: usize, height: usize) -> Vec<HeatmapVertex> {
        let mut vertices = Vec::with_capacity(width * height * 6);
        let scale_x = 2.0;
        let scale_y = 2.0;

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let val = data.get(idx).copied().unwrap_or(0.0);

                let x0 = -1.0 + (x as f32 / width as f32) * scale_x;
                let x1 = -1.0 + ((x + 1) as f32 / width as f32) * scale_x;

                let y0 = 1.0 - (y as f32 / height as f32) * scale_y;
                let y1 = 1.0 - ((y + 1) as f32 / height as f32) * scale_y;

                let u0 = x as f32 / width as f32;
                let u1 = (x + 1) as f32 / width as f32;
                let v0 = y as f32 / height as f32;
                let v1 = (y + 1) as f32 / height as f32;

                let v_tl = HeatmapVertex { position: [x0, y0], uv: [u0, v0], val };
                let v_tr = HeatmapVertex { position: [x1, y0], uv: [u1, v0], val };
                let v_bl = HeatmapVertex { position: [x0, y1], uv: [u0, v1], val };
                let v_br = HeatmapVertex { position: [x1, y1], uv: [u1, v1], val };

                vertices.push(v_tl);
                vertices.push(v_bl);
                vertices.push(v_tr);

                vertices.push(v_tr);
                vertices.push(v_bl);
                vertices.push(v_br);
            }
        }

        vertices
    }
}

pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub colormap: u32,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for HeatmapCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.renderer.update_colormap(queue, self.colormap);
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
        rpass.draw(0..self.renderer.num_vertices, 0..1);
    }
}

// Backward compatibility alias during refactoring
pub type MatrixRenderer = HeatmapRenderer;
pub type MatrixCallback = HeatmapCallback;
