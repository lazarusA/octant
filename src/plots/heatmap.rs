use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
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
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HeatmapUniforms {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub _pad: u32,
    pub aspect_scale: [f32; 2],
    pub width: u32,
    pub height: u32,
    pub color: super::common::PlotColorParams,
}

use std::sync::atomic::{AtomicU32, Ordering};

pub struct HeatmapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
    width: AtomicU32,
    height: AtomicU32,
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
            pan: [0.0, 0.0],
            zoom: 1.0,
            _pad: 0,
            aspect_scale: [1.0, 1.0],
            width: width.max(1) as u32,
            height: height.max(1) as u32,
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Heatmap Uniform Buffer",
            &initial_uniforms,
        );

        let capacity_elements = (width * height).clamp(
            2048 * 2048,
            crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS,
        );
        let mut padded_initial = matrix_data.to_vec();
        if padded_initial.len() < capacity_elements {
            padded_initial.resize(capacity_elements, 0.0);
        }

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Heatmap Data Storage Buffer",
            &padded_initial,
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Heatmap Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(HeatmapVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(super::common::default_depth_stencil_state(
                false,
                wgpu::CompareFunction::Always,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let (vertices, indices) = Self::build_quad();

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
            width: AtomicU32::new(width as u32),
            height: AtomicU32::new(height as u32),
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        color: &super::common::PlotColorParams,
        pan: [f32; 2],
        zoom: f32,
        aspect_scale: [f32; 2],
    ) {
        let uniforms = HeatmapUniforms {
            pan,
            zoom,
            _pad: 0,
            aspect_scale,
            width: self.width.load(Ordering::Relaxed),
            height: self.height.load(Ordering::Relaxed),
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_colormap(&self, queue: &wgpu::Queue, colormap: u32) {
        let color = super::common::PlotColorParams {
            colormap,
            ..Default::default()
        };
        self.update_uniforms(queue, &color, [0.0, 0.0], 1.0, [1.0, 1.0]);
    }

    /// Fast GPU Storage Buffer data channel upload
    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    /// Updates data and dimensions for dynamic viewport LOD resampling
    pub fn update_data_and_dimensions(
        &self,
        queue: &wgpu::Queue,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) {
        self.width.store(width as u32, Ordering::Relaxed);
        self.height.store(height as u32, Ordering::Relaxed);
        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    fn build_quad() -> (Vec<HeatmapVertex>, Vec<u32>) {
        let vertices = vec![
            HeatmapVertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
            HeatmapVertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            HeatmapVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            HeatmapVertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
        ];

        let indices = vec![0, 2, 1, 1, 2, 3];

        (vertices, indices)
    }
}

impl super::common::PlotRenderer for HeatmapRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
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
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.pan,
            self.zoom,
            self.aspect_scale,
        );
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        super::common::setup_viewport_and_scissor(rpass, &self.rect, &info);

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
