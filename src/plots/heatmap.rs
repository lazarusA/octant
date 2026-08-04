use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub cell_index: u32,
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
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapUniforms {
    pub color: super::common::PlotColorParams,
}

pub struct HeatmapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
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
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Heatmap Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Heatmap Data Storage Buffer",
            matrix_data,
        );

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Heatmap Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Heatmap Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            label: Some("Heatmap Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Heatmap Index Buffer"),
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

    pub fn update_uniforms(&self, queue: &wgpu::Queue, color: &super::common::PlotColorParams) {
        let uniforms = HeatmapUniforms { color: *color };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_colormap(&self, queue: &wgpu::Queue, colormap: u32) {
        let mut color = super::common::PlotColorParams::default();
        color.colormap = colormap;
        self.update_uniforms(queue, &color);
    }

    /// Fast GPU Storage Buffer data channel upload
    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    fn build_mesh(width: usize, height: usize) -> (Vec<HeatmapVertex>, Vec<u32>) {
        let num_quads = width * height;
        let mut vertices = Vec::with_capacity(num_quads * 4);
        let mut indices = Vec::with_capacity(num_quads * 6);

        let scale_x = 2.0;
        let scale_y = 2.0;

        for y in 0..height {
            for x in 0..width {
                let cell_index = (y * width + x) as u32;

                let x0 = -1.0 + (x as f32 / width as f32) * scale_x;
                let x1 = -1.0 + ((x + 1) as f32 / width as f32) * scale_x;

                let y0 = 1.0 - (y as f32 / height as f32) * scale_y;
                let y1 = 1.0 - ((y + 1) as f32 / height as f32) * scale_y;

                let u0 = x as f32 / width as f32;
                let u1 = (x + 1) as f32 / width as f32;
                let v0 = y as f32 / height as f32;
                let v1 = (y + 1) as f32 / height as f32;

                let base_idx = vertices.len() as u32;

                vertices.push(HeatmapVertex {
                    position: [x0, y0],
                    uv: [u0, v0],
                    cell_index,
                });
                vertices.push(HeatmapVertex {
                    position: [x1, y0],
                    uv: [u1, v0],
                    cell_index,
                });
                vertices.push(HeatmapVertex {
                    position: [x0, y1],
                    uv: [u0, v1],
                    cell_index,
                });
                vertices.push(HeatmapVertex {
                    position: [x1, y1],
                    uv: [u1, v1],
                    cell_index,
                });

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

pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
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
        self.renderer.update_uniforms(queue, &self.color_params);
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
        rpass.set_index_buffer(
            self.renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.draw_indexed(0..self.renderer.num_indices, 0, 0..1);
    }
}

// Backward compatibility alias during refactoring
pub type MatrixRenderer = HeatmapRenderer;
pub type MatrixCallback = HeatmapCallback;
