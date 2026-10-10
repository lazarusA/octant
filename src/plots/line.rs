use bytemuck::{Pod, Zeroable};
use eframe::egui;
use std::sync::Arc;
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
    pub point_size: f32,
    pub use_custom_color: u32,
    pub show_lines: u32,
    pub show_points: u32,
    pub screen_aspect: f32,
    pub _pad0: [u32; 2],
    pub line_color: [f32; 4],
    pub color: super::common::PlotColorParams,
}

pub struct LineUniformParams {
    pub color: super::common::PlotColorParams,
    pub line_color: [f32; 4],
    pub use_custom_color: bool,
    pub show_lines: bool,
    pub show_points: bool,
    pub point_size: f32,
    pub screen_aspect: f32,
    pub viewport_padding: [f32; 2],
    pub profile_length: u32,
    pub line_count: u32,
    pub line_mode: u32,
    pub pan: [f32; 2],
    pub zoom: f32,
}

use std::sync::RwLock;

use super::line_payload::{LineShape, PayloadSlot, buffer_capacity};

pub struct LineRenderer {
    render_pipeline: wgpu::RenderPipeline,
    scatter_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    gpu_resources: RwLock<LineGpuResources>,
    /// The payload the data buffer holds (`upload_payload`).
    payload: PayloadSlot,
}

struct LineGpuResources {
    data_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl LineRenderer {
    /// A renderer with an empty data buffer: the first paint uploads the
    /// line payload (`upload_payload`).
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/line.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("1D Line & Scatter WGSL Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1D Line Storage Buffer"),
            contents: bytemuck::cast_slice(&[0u32; 64]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let initial_uniforms = LineUniforms {
            viewport_padding: [0.08, 0.12], // 8% horizontal, 12% vertical dynamic padding
            line_thickness: 2.0,
            profile_length: 1,
            line_count: 1,
            line_mode: 0,
            pan: [0.0, 0.0],
            zoom: 1.0,
            point_size: 6.0,
            use_custom_color: 1,
            show_lines: 1,
            show_points: 0,
            screen_aspect: 1.0,
            _pad0: [0; 2],
            line_color: [0.2, 0.65, 1.0, 1.0],
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

        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("1D Line Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&colormap_layout)],
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

        let scatter_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("1D Scatter Points Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_scatter"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_scatter"),
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
            scatter_pipeline,
            bind_group_layout,
            uniform_buffer,
            gpu_resources: RwLock::new(LineGpuResources {
                data_buffer,
                bind_group,
            }),
            payload: PayloadSlot::default(),
        }
    }

    /// The shape of the payload for `key`, when that is what the buffer holds.
    pub fn payload_shape(&self, key: u64) -> Option<LineShape> {
        self.payload.shape(key)
    }

    /// Uploads `words`, a payload of `shape` (`line_payload`), as the one for
    /// `key`. A payload past the device's buffer limit draws nothing.
    pub fn upload_payload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: u64,
        words: &[u32],
        shape: LineShape,
    ) {
        let uploaded =
            words.is_empty() || self.write_data(device, queue, bytemuck::cast_slice(words));
        let drawn_lines = if uploaded { shape.drawn_lines } else { 0 };
        // Held either way, so a payload that does not fit is reported once.
        self.payload.hold(
            key,
            LineShape {
                drawn_lines,
                ..shape
            },
        );
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
            point_size: params.point_size,
            use_custom_color: if params.use_custom_color { 1 } else { 0 },
            show_lines: if params.show_lines { 1 } else { 0 },
            show_points: if params.show_points { 1 } else { 0 },
            screen_aspect: params.screen_aspect,
            _pad0: [0; 2],
            line_color: params.line_color,
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Writes `bytes` to the start of the data buffer, growing it (to the next
    /// power of two, within the device limit) when they do not fit. Reports
    /// and returns `false` when they exceed the device's storage buffer limit.
    fn write_data(&self, device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> bool {
        let needed = bytes.len() as u64;
        let current = self
            .gpu_resources
            .read()
            .map(|g| g.data_buffer.size())
            .unwrap_or(0);
        if needed <= current {
            if let Ok(guard) = self.gpu_resources.read() {
                queue.write_buffer(&guard.data_buffer, 0, bytes);
            }
            return true;
        }
        let limits = device.limits();
        let limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        let Some(capacity) = buffer_capacity(needed, limit) else {
            crate::ui::toast::report(
                crate::ui::toast::Severity::Warning,
                "Line plot too large for the GPU",
                format!("The lines take {needed} bytes; this GPU binds at most {limit}."),
            );
            return false;
        };
        let data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("1D Line Storage Buffer (Resized)"),
            size: capacity,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "1D Line Bind Group (Resized)",
            &self.bind_group_layout,
            &self.uniform_buffer,
            &data_buffer,
        );
        queue.write_buffer(&data_buffer, 0, bytes);
        if let Ok(mut guard) = self.gpu_resources.write() {
            guard.data_buffer = data_buffer;
            guard.bind_group = bind_group;
        }
        true
    }

    /// Writes raw values into the data buffer (the `PlotRenderer` traits);
    /// the next paint uploads its payload again.
    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        if matrix_data.is_empty() {
            return;
        }
        self.payload.forget();
        if let Ok(guard) = self.gpu_resources.read() {
            super::common::safe_write_buffer(
                queue,
                &guard.data_buffer,
                matrix_data,
                "LineRenderer::update_data",
            );
        }
    }
}

impl super::common::PlotRenderer for LineRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

impl super::traits::PlotRenderer for LineRenderer {
    fn update_data(&self, queue: &wgpu::Queue, data: &crate::data::RenderData) {
        if let crate::data::RenderData::Matrix(m) = data {
            self.update_data(queue, &m.values);
        }
    }

    fn paint(
        &self,
        _ui: &mut egui::Ui,
        _rect: egui::Rect,
        _params: &super::traits::PlotRenderParams,
    ) {
        // Concrete painter dispatched via egui callback
    }

    fn inspect_hover(
        &self,
        _pointer_pos: egui::Pos2,
        _rect: egui::Rect,
        _data: &crate::data::RenderData,
        _params: &super::traits::PlotRenderParams,
    ) -> Option<super::traits::HoverSample> {
        None
    }
}

pub struct LineCallback {
    pub renderer: Arc<LineRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub line_color: [f32; 4],
    pub use_custom_color: bool,
    pub show_lines: bool,
    pub show_points: bool,
    pub point_size: f32,
    pub rect: egui::Rect,
    /// The payload uploaded to the renderer (`LineRenderer::upload_payload`).
    pub shape: LineShape,
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
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        super::colormap_atlas::prepare(device, queue, callback_resources);
        let screen_aspect = self.rect.width() / self.rect.height().max(1.0);
        self.renderer.update_uniforms(
            queue,
            &LineUniformParams {
                color: self.color_params,
                line_color: self.line_color,
                use_custom_color: self.use_custom_color,
                show_lines: self.show_lines,
                show_points: self.show_points,
                point_size: self.point_size,
                screen_aspect,
                viewport_padding: [0.0, 0.0],
                profile_length: self.shape.profile_length,
                line_count: self.shape.line_count,
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
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if !super::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }
        if !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }

        let Ok(guard) = self.renderer.gpu_resources.read() else {
            return;
        };
        rpass.set_bind_group(0, &guard.bind_group, &[]);

        let LineShape {
            profile_length,
            drawn_lines: line_count,
            ..
        } = self.shape;
        if line_count > 0 {
            // A single sample has no segment: only its point shows.
            if self.show_lines && profile_length >= 2 {
                rpass.set_pipeline(&self.renderer.render_pipeline);
                rpass.draw(0..profile_length, 0..line_count);
            }
            if self.show_points {
                // One point per sample: the shader splits instances by the
                // uniform `profile_length` (at least 1).
                let points = profile_length.max(1) * line_count;
                rpass.set_pipeline(&self.renderer.scatter_pipeline);
                rpass.draw(0..6, 0..points);
            }
        }
    }
}
