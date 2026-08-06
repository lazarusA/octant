use bytemuck::{Pod, Zeroable};
use eframe::egui;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineVertex {
    pub position: [f32; 2],
    pub cell_index: u32,
}

impl LineVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineUniforms {
    pub viewport_padding: [f32; 2],
    pub line_thickness: f32,
    pub _pad0: u32,
    pub color: super::common::PlotColorParams,
}

pub struct LineRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_vertices: u32,
}

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/line.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("1D Line WGSL Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let safe_data = if matrix_data.is_empty() {
            vec![0.0f32; 64]
        } else {
            matrix_data.to_vec()
        };

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1D Line Storage Buffer"),
            contents: bytemuck::cast_slice(&safe_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let initial_uniforms = LineUniforms {
            viewport_padding: [0.08, 0.12], // 8% horizontal, 12% vertical dynamic padding
            line_thickness: 2.0,
            _pad0: 0,
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1D Line Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "1D Line Bind Group Layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "1D Line Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("1D Line Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("1D Line Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(LineVertex::desc())],
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
                topology: wgpu::PrimitiveTopology::LineStrip,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertices = Self::build_line_vertices(&safe_data, width, height);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1D Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            data_buffer,
            uniform_buffer,
            bind_group,
            num_vertices: vertices.len() as u32,
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        color: &super::common::PlotColorParams,
        padding_x: f32,
        padding_y: f32,
    ) {
        let uniforms = LineUniforms {
            viewport_padding: [padding_x.clamp(0.02, 0.2), padding_y.clamp(0.02, 0.3)],
            line_thickness: 2.5,
            _pad0: 0,
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        if !matrix_data.is_empty() {
            queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
        }
    }

    fn build_line_vertices(values: &[f32], _width: usize, _height: usize) -> Vec<LineVertex> {
        let n = values.len();
        if n == 0 {
            return Vec::new();
        }

        let count = n.max(2);
        let mut vertices = Vec::with_capacity(count);

        for (i, _) in values.iter().enumerate() {
            let norm_x = if count > 1 {
                (i as f32 / (count - 1) as f32) * 2.0 - 1.0
            } else {
                0.0
            };

            vertices.push(LineVertex {
                position: [norm_x, 0.0],
                cell_index: i as u32,
            });
        }

        vertices
    }
}

impl super::common::PlotRenderer for LineRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        LineRenderer::update_data(self, queue, values);
    }
}

pub struct LineCallback {
    pub renderer: Arc<LineRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for LineCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Dynamic viewport padding computed from canvas dimensions
        let padding_x = (40.0 / self.rect.width().max(1.0)).clamp(0.04, 0.15);
        let padding_y = (35.0 / self.rect.height().max(1.0)).clamp(0.06, 0.20);
        self.renderer
            .update_uniforms(queue, &self.color_params, padding_x, padding_y);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        super::common::setup_viewport_and_scissor(rpass, &self.rect, info.pixels_per_point);

        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
        rpass.draw(0..self.renderer.num_vertices, 0..1);
    }
}
