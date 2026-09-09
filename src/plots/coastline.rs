//! GPU coastline overlay renderer.
//!
//! Draws Natural Earth coastline line-strips on top of 2D Heatmap / Flatmap plots
//! using a vertex-pull approach: lon/lat pairs are stored in a GPU storage buffer
//! and indexed by `vertex_index` in the WGSL shader.
//!
//! # LOD hot-swap
//!
//! Call [`CoastlineRenderer::swap_vertices`] with new `&[f32]` data (from the
//! background tokio task) to upgrade the coastline resolution while the app is
//! running. If the new data fits in the existing buffer it is written in-place;
//! otherwise the buffer is reallocated.

use bytemuck::{Pod, Zeroable};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;

// ---------------------------------------------------------------------------
// Uniform buffer layout — must match coastline.wgsl `CoastlineUniforms`
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CoastlineUniforms {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub _pad0: u32,
    pub aspect_scale: [f32; 2],
    pub _pad1: u32,
    pub _pad2: u32,
    pub line_color: [f32; 4],
    /// Dataset geographic bounds in degrees.
    /// `lon_min`/`lon_max`: western/eastern edge (e.g. `0`/`360` or `-180`/`180`).
    /// `lat_min`/`lat_max`: stored in dataset storage order — `lat_min > lat_max`
    /// means the dataset is north-down (row 0 = 90°N).
    pub lon_min: f32,
    pub lon_max: f32,
    pub lat_min: f32,
    pub lat_max: f32,
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

struct CoastlineGpuResources {
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

pub struct CoastlineRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    gpu: RwLock<CoastlineGpuResources>,
    vertex_count: AtomicU32,
}

impl CoastlineRenderer {
    /// Creates the pipeline and uploads the initial coastline vertices.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat, verts: &[f32]) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Coastline Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/coastline.wgsl").into()),
        });

        let initial_uniforms = CoastlineUniforms {
            pan: [0.0; 2],
            zoom: 1.0,
            _pad0: 0,
            aspect_scale: [1.0; 2],
            _pad1: 0,
            _pad2: 0,
            line_color: [1.0, 1.0, 1.0, 0.8], // default: semi-transparent white
            // Global defaults — overwritten each frame from the active dataset grid.
            lon_min: -180.0,
            lon_max: 180.0,
            lat_min: -90.0, // southern edge (canonical min ≤ max)
            lat_max: 90.0,
        };

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Coastline Uniform Buffer",
            &initial_uniforms,
        );

        let safe_verts: &[f32] = if verts.is_empty() {
            &[0.0_f32, 0.0]
        } else {
            verts
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Coastline Vertex Buffer"),
            contents: bytemuck::cast_slice(safe_verts),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Coastline Bind Group Layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Coastline Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &vertex_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Coastline Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Coastline Render Pipeline"),
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

        let vertex_count = (safe_verts.len() / 2) as u32;

        Self {
            render_pipeline,
            bind_group_layout,
            uniform_buffer,
            gpu: RwLock::new(CoastlineGpuResources {
                vertex_buffer,
                bind_group,
            }),
            vertex_count: AtomicU32::new(vertex_count),
        }
    }

    /// Updates the uniform buffer each frame.
    #[allow(clippy::too_many_arguments)]
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        pan: [f32; 2],
        zoom: f32,
        aspect_scale: [f32; 2],
        line_color: [f32; 4],
        lon_min: f32,
        lon_max: f32,
        lat_min: f32,
        lat_max: f32,
    ) {
        let uniforms = CoastlineUniforms {
            pan,
            zoom,
            _pad0: 0,
            aspect_scale,
            _pad1: 0,
            _pad2: 0,
            line_color,
            lon_min,
            lon_max,
            lat_min,
            lat_max,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Hot-swaps the vertex buffer with higher/lower LOD data.
    ///
    /// If the new data fits in the existing buffer it is written in-place (no
    /// allocation). Otherwise the buffer is reallocated and the bind group is
    /// recreated.
    pub fn swap_vertices(&self, device: &wgpu::Device, queue: &wgpu::Queue, verts: &[f32]) {
        if verts.is_empty() {
            return;
        }
        let needed = std::mem::size_of_val(verts) as u64;
        let current_capacity = self.gpu.read().map(|g| g.vertex_buffer.size()).unwrap_or(0);

        if needed <= current_capacity {
            if let Ok(guard) = self.gpu.read() {
                queue.write_buffer(&guard.vertex_buffer, 0, bytemuck::cast_slice(verts));
            }
        } else {
            let new_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Coastline Vertex Buffer (Resized)"),
                contents: bytemuck::cast_slice(verts),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let new_bg = super::common::create_uniform_storage_bind_group(
                device,
                "Coastline Bind Group (Resized)",
                &self.bind_group_layout,
                &self.uniform_buffer,
                &new_buf,
            );
            if let Ok(mut guard) = self.gpu.write() {
                guard.vertex_buffer = new_buf;
                guard.bind_group = new_bg;
            }
        }

        self.vertex_count
            .store((verts.len() / 2) as u32, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------
// egui-wgpu paint callback
// ---------------------------------------------------------------------------

pub struct CoastlineCallback {
    pub renderer: Arc<CoastlineRenderer>,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub line_color: [f32; 4],
    pub rect: egui::Rect,
    /// Dataset geographic bounds in degrees; forwarded to `CoastlineUniforms`.
    pub lon_min: f32,
    pub lon_max: f32,
    pub lat_min: f32,
    pub lat_max: f32,
}

impl eframe::egui_wgpu::CallbackTrait for CoastlineCallback {
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
            self.pan,
            self.zoom,
            self.aspect_scale,
            self.line_color,
            self.lon_min,
            self.lon_max,
            self.lat_min,
            self.lat_max,
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
        let vertex_count = self.renderer.vertex_count.load(Ordering::Relaxed);
        if vertex_count < 2 {
            return;
        }
        rpass.set_pipeline(&self.renderer.render_pipeline);
        let Ok(guard) = self.renderer.gpu.read() else {
            return;
        };
        rpass.set_bind_group(0, &guard.bind_group, &[]);
        rpass.draw(0..vertex_count, 0..1);
    }
}
