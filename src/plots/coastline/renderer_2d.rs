//! 2D GPU coastline overlay renderer for flatmaps and heatmaps.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;

use super::expansion::expand_coastline_line_list;
use super::types::CoastlineUniforms;

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
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat, verts: &[f32]) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Coastline 2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/coastline.wgsl").into()),
        });

        let initial_uniforms = CoastlineUniforms {
            pan: [0.0; 2],
            zoom: 1.0,
            crop_to_domain: 1,
            aspect_scale: [1.0; 2],
            line_width: 1.0,
            _pad2: 0,
            line_color: [1.0, 1.0, 1.0, 0.8],
            lon_min: -180.0,
            lon_max: 180.0,
            lat_min: -90.0,
            lat_max: 90.0,
        };

        let uniform_buffer = crate::plots::common::create_uniform_buffer(
            device,
            "Coastline 2D Uniform Buffer",
            &initial_uniforms,
        );

        let line_vertices = expand_coastline_line_list(verts);
        let safe_verts: &[f32] = if line_vertices.is_empty() {
            &[0.0_f32, 0.0]
        } else {
            line_vertices.as_slice()
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Coastline 2D Vertex Buffer"),
            contents: bytemuck::cast_slice(safe_verts),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = crate::plots::common::create_uniform_storage_bind_group_layout(
            device,
            "Coastline 2D Bind Group Layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let bind_group = crate::plots::common::create_uniform_storage_bind_group(
            device,
            "Coastline 2D Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &vertex_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Coastline 2D Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Coastline 2D Render Pipeline"),
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
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(crate::plots::common::default_depth_stencil_state(
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
            gpu: RwLock::new(CoastlineGpuResources {
                vertex_buffer,
                bind_group,
            }),
            vertex_count: AtomicU32::new((safe_verts.len() / 2) as u32),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        pan: [f32; 2],
        zoom: f32,
        crop_to_domain: bool,
        aspect_scale: [f32; 2],
        line_color: [f32; 4],
        line_width: f32,
        lon_min: f32,
        lon_max: f32,
        lat_min: f32,
        lat_max: f32,
    ) {
        let uniforms = CoastlineUniforms {
            pan,
            zoom,
            crop_to_domain: if crop_to_domain { 1 } else { 0 },
            aspect_scale,
            line_width,
            _pad2: 0,
            line_color,
            lon_min,
            lon_max,
            lat_min,
            lat_max,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn swap_vertices(&self, device: &wgpu::Device, queue: &wgpu::Queue, verts: &[f32]) {
        let line_vertices = expand_coastline_line_list(verts);
        if line_vertices.is_empty() {
            return;
        }
        let needed = std::mem::size_of_val(line_vertices.as_slice()) as u64;
        let current_capacity = self.gpu.read().map(|g| g.vertex_buffer.size()).unwrap_or(0);

        if needed <= current_capacity {
            if let Ok(guard) = self.gpu.read() {
                queue.write_buffer(
                    &guard.vertex_buffer,
                    0,
                    bytemuck::cast_slice(line_vertices.as_slice()),
                );
            }
        } else {
            let new_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Coastline 2D Vertex Buffer (Resized)"),
                contents: bytemuck::cast_slice(line_vertices.as_slice()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let new_bg = crate::plots::common::create_uniform_storage_bind_group(
                device,
                "Coastline 2D Bind Group (Resized)",
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
            .store((line_vertices.len() / 2) as u32, Ordering::Relaxed);
    }
}

pub struct CoastlineCallback {
    pub renderer: Arc<CoastlineRenderer>,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub crop_to_domain: bool,
    pub aspect_scale: [f32; 2],
    pub line_color: [f32; 4],
    pub line_width: f32,
    pub rect: egui::Rect,
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
            self.crop_to_domain,
            self.aspect_scale,
            self.line_color,
            self.line_width,
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
        if !crate::plots::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
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
        let instances = (self.line_width.round() as u32 * 2)
            .saturating_sub(1)
            .clamp(1, 7);
        rpass.draw(0..vertex_count, 0..instances);
    }
}
