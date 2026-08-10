use bytemuck::{Pod, Zeroable};
use eframe::egui;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineVertex {
    pub position: [f32; 2],
    pub cell_index: u32,
    pub line_index: u32,
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
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
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
    pub profile_length: u32,
    pub line_count: u32,
    pub line_mode: u32,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub _pad1: u32,
    /// WGSL aligns nested `ColorUniforms` to 16 bytes (offset 48).
    pub _color_align_pad: [u32; 2],
    pub color: super::common::PlotColorParams,
}

pub struct LineUniformParams {
    pub color: super::common::PlotColorParams,
    pub viewport_padding: [f32; 2],
    pub profile_length: u32,
    pub line_count: u32,
    pub line_mode: u32,
    pub pan: [f32; 2],
    pub zoom: f32,
}

pub struct LineRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_vertices: AtomicU32,
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
            profile_length: width.max(1) as u32,
            line_count: height.max(1) as u32,
            line_mode: 0,
            pan: [0.0, 0.0],
            zoom: 1.0,
            _pad1: 0,
            _color_align_pad: [0; 2],
            color: super::common::PlotColorParams::default(),
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "1D Line Uniform Buffer",
            &initial_uniforms,
        );

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

        let initial_vertices = Self::build_line_vertices(width.max(1), height.max(1));
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1D Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&initial_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            data_buffer,
            uniform_buffer,
            bind_group,
            num_vertices: AtomicU32::new(initial_vertices.len() as u32),
        }
    }

    pub fn update_profile_geometry(
        &self,
        queue: &wgpu::Queue,
        profile_length: u32,
        line_count: u32,
    ) {
        let profile_length = profile_length.max(1) as usize;
        let line_count = line_count.max(1) as usize;
        let vertices = Self::build_line_vertices(profile_length, line_count);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.num_vertices
            .store(vertices.len() as u32, Ordering::Relaxed);
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &LineUniformParams) {
        let uniforms = LineUniforms {
            viewport_padding: [
                params.viewport_padding[0].clamp(0.02, 0.2),
                params.viewport_padding[1].clamp(0.02, 0.3),
            ],
            line_thickness: 2.5,
            profile_length: params.profile_length.max(1),
            line_count: params.line_count.max(1),
            line_mode: params.line_mode,
            pan: params.pan,
            zoom: params.zoom,
            _pad1: 0,
            _color_align_pad: [0; 2],
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        if matrix_data.is_empty() {
            return;
        }

        let needed_bytes = std::mem::size_of_val(matrix_data) as u64;
        let capacity_bytes = self.data_buffer.size();

        if needed_bytes > capacity_bytes {
            // Should be unreachable given current state-reset invariants, but this
            // guards against future state-sync bugs causing a hard wgpu panic.
            log::error!(
                "LineRenderer::update_data: payload ({needed_bytes} bytes) exceeds \
                 buffer capacity ({capacity_bytes} bytes) - dropping update. This \
                 usually means matrix_data.len() doesn't match the (width, height) \
                 the renderer was created with."
            );
            return;
        }

        queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(matrix_data));
    }

    fn build_line_vertices(profile_length: usize, line_count: usize) -> Vec<LineVertex> {
        let profile_length = profile_length.max(2);
        let line_count = line_count.max(1);
        let mut vertices = Vec::with_capacity(profile_length * line_count);

        for line_idx in 0..line_count {
            for point_idx in 0..profile_length {
                let norm_x = if profile_length > 1 {
                    (point_idx as f32 / (profile_length - 1) as f32) * 2.0 - 1.0
                } else {
                    0.0
                };

                vertices.push(LineVertex {
                    position: [norm_x, 0.0],
                    cell_index: point_idx as u32,
                    line_index: line_idx as u32,
                });
            }
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
    pub profile_values: Vec<f32>,
    pub profile_length: u32,
    pub line_count: u32,
    pub line_mode: u32,
    pub pan: [f32; 2],
    pub zoom: f32,
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

        if !self.profile_values.is_empty() {
            self.renderer.update_data(queue, &self.profile_values);
        }
        self.renderer
            .update_profile_geometry(queue, self.profile_length, self.line_count);
        self.renderer.update_uniforms(
            queue,
            &LineUniformParams {
                color: self.color_params,
                viewport_padding: [padding_x, padding_y],
                profile_length: self.profile_length,
                line_count: self.line_count,
                line_mode: self.line_mode,
                pan: self.pan,
                zoom: self.zoom,
            },
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

        let profile_length = self.profile_length.max(2);
        let line_count = self.line_count.max(1);
        for line_idx in 0..line_count {
            let start = line_idx * profile_length;
            let end = start + profile_length;
            rpass.draw(start..end, 0..1);
        }
    }
}
