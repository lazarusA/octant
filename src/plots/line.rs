use bytemuck::{Pod, Zeroable};
use eframe::egui;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::util::DeviceExt;

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

use std::sync::RwLock;

pub struct LineRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    gpu_resources: RwLock<LineGpuResources>,
    data_len: AtomicU32,
}

struct LineGpuResources {
    data_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
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
                buffers: &[],
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
            depth_stencil: Some(super::common::default_depth_stencil_state(
                false,
                wgpu::CompareFunction::Always,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            uniform_buffer,
            gpu_resources: RwLock::new(LineGpuResources {
                data_buffer,
                bind_group,
            }),
            data_len: AtomicU32::new(safe_data.len() as u32),
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &LineUniformParams) {
        let uniforms = LineUniforms {
            viewport_padding: params.viewport_padding,
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

    pub fn update_data_with_device(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        matrix_data: &[f32],
    ) {
        if matrix_data.is_empty() {
            return;
        }

        let needed_bytes = std::mem::size_of_val(matrix_data) as u64;
        let current_capacity = self
            .gpu_resources
            .read()
            .map(|g| g.data_buffer.size())
            .unwrap_or(0);

        if needed_bytes > current_capacity {
            let new_capacity = needed_bytes.next_power_of_two();
            let new_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("1D Line Storage Buffer (Resized)"),
                size: new_capacity,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let new_bind_group = super::common::create_uniform_storage_bind_group(
                device,
                "1D Line Bind Group (Resized)",
                &self.bind_group_layout,
                &self.uniform_buffer,
                &new_data_buffer,
            );

            queue.write_buffer(&new_data_buffer, 0, bytemuck::cast_slice(matrix_data));

            if let Ok(mut guard) = self.gpu_resources.write() {
                guard.data_buffer = new_data_buffer;
                guard.bind_group = new_bind_group;
            }
        } else if let Ok(guard) = self.gpu_resources.read() {
            queue.write_buffer(&guard.data_buffer, 0, bytemuck::cast_slice(matrix_data));
        }

        self.data_len
            .store(matrix_data.len() as u32, Ordering::Relaxed);
    }

    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        if matrix_data.is_empty() {
            return;
        }

        if let Ok(guard) = self.gpu_resources.read()
            && super::common::safe_write_buffer(
                queue,
                &guard.data_buffer,
                matrix_data,
                "LineRenderer::update_data",
            )
        {
            self.data_len
                .store(matrix_data.len() as u32, Ordering::Relaxed);
        }
    }

    /// Renders the 1D line chart series directly into the active render pass.
    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'_>, profile_length: u32, line_count: u32) {
        rpass.set_pipeline(&self.render_pipeline);
        let Ok(guard) = self.gpu_resources.read() else {
            return;
        };
        rpass.set_bind_group(0, &guard.bind_group, &[]);
        let profile_length = profile_length.max(2);
        if line_count > 0 {
            rpass.draw(0..profile_length, 0..line_count);
        }
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if !self.profile_values.is_empty() {
            self.renderer
                .update_data_with_device(device, queue, &self.profile_values);
        }
        self.renderer.update_uniforms(
            queue,
            &LineUniformParams {
                color: self.color_params,
                viewport_padding: [0.0, 0.0],
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
        if !super::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }

        rpass.set_pipeline(&self.renderer.render_pipeline);
        let Ok(guard) = self.renderer.gpu_resources.read() else {
            return;
        };
        rpass.set_bind_group(0, &guard.bind_group, &[]);

        let profile_length = self.profile_length.max(2);
        let line_count = self.line_count;
        if line_count > 0 && !self.profile_values.is_empty() {
            rpass.draw(0..profile_length, 0..line_count);
        }
    }
}
