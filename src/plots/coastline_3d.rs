//! GPU coastline overlay renderer for 3D surfaces and spherical plots.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;

use super::common::safe_write_buffer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Coastline3DUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub plot_kind: u32,
    pub plot_mode: u32,
    pub width: u32,
    pub height: u32,
    pub coord_mode: u32,
    pub _pad_bounds: [u32; 2],
    pub lon_bounds: [f32; 2],
    pub lat_bounds: [f32; 2],
    pub color: [f32; 4],
    pub _pad_color: [u32; 2],
    pub color_range: [f32; 2],
    pub _pad: [u32; 2],
    pub _pad_tail: [u32; 2],
}

struct Coastline3DGpuResources {
    coastline_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

pub struct Coastline3DRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    coord_x_buffer: wgpu::Buffer,
    coord_y_buffer: wgpu::Buffer,
    gpu: RwLock<Coastline3DGpuResources>,
    vertex_count: AtomicU32,
    width: u32,
    height: u32,
}

impl Coastline3DRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        coastline_vertices: &[f32],
        values: &[f32],
        width: usize,
        height: usize,
        coords_x: Option<&[f32]>,
        coords_y: Option<&[f32]>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Coastline Shader"),
            source: wgpu::ShaderSource::Wgsl(
                crate::assemble_plot_with_coords_shader!(include_str!("shaders/coastline_3d.wgsl"))
                    .into(),
            ),
        });

        let uniforms = Coastline3DUniforms {
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_ratio: 1.0,
            zoom: 2.5,
            displacement_strength: 0.0,
            plot_kind: 0,
            plot_mode: 0,
            width: width as u32,
            height: height as u32,
            coord_mode: 0,
            _pad_bounds: [0; 2],
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            color: [1.0, 1.0, 1.0, 0.8],
            _pad_color: [0; 2],
            color_range: [-1.0, 1.0],
            _pad: [0; 2],
            _pad_tail: [0; 2],
        };
        let uniform_buffer =
            super::common::create_uniform_buffer(device, "3D Coastline Uniform Buffer", &uniforms);
        let data_buffer = super::common::create_storage_buffer(
            device,
            "3D Coastline Data Storage Buffer",
            if values.is_empty() { &[0.0] } else { values },
        );

        let dummy_coords = [0.0_f32; 1];
        let mut x_coords = coords_x
            .filter(|coords| !coords.is_empty())
            .unwrap_or(&dummy_coords)
            .to_vec();
        x_coords.resize(x_coords.len().max(width.max(128)), 0.0);
        let mut y_coords = coords_y
            .filter(|coords| !coords.is_empty())
            .unwrap_or(&dummy_coords)
            .to_vec();
        y_coords.resize(y_coords.len().max(height.max(128)), 0.0);
        let coord_x_buffer = super::common::create_storage_buffer(
            device,
            "3D Coastline Coord X Storage Buffer",
            &x_coords,
        );
        let coord_y_buffer = super::common::create_storage_buffer(
            device,
            "3D Coastline Coord Y Storage Buffer",
            &y_coords,
        );

        let safe_vertices = if coastline_vertices.is_empty() {
            &[0.0_f32, 0.0][..]
        } else {
            coastline_vertices
        };
        let coastline_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("3D Coastline Vertex Buffer"),
            contents: bytemuck::cast_slice(safe_vertices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("3D Coastline Bind Group Layout"),
            entries: &[
                buffer_layout(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    wgpu::BufferBindingType::Uniform,
                ),
                buffer_layout(
                    1,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                buffer_layout(
                    2,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                buffer_layout(
                    3,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                buffer_layout(
                    4,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            ],
        });
        let bind_group = make_bind_group(
            device,
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
            &coord_x_buffer,
            &coord_y_buffer,
            &coastline_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Coastline Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Coastline Render Pipeline"),
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
                wgpu::CompareFunction::LessEqual,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            uniform_buffer,
            data_buffer,
            coord_x_buffer,
            coord_y_buffer,
            gpu: RwLock::new(Coastline3DGpuResources {
                coastline_buffer,
                bind_group,
            }),
            vertex_count: AtomicU32::new((safe_vertices.len() / 2) as u32),
            width: width as u32,
            height: height as u32,
        }
    }

    pub fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        safe_write_buffer(
            queue,
            &self.data_buffer,
            values,
            "Coastline3DRenderer::update_data",
        );
    }

    pub fn update_coords(&self, queue: &wgpu::Queue, coords_x: &[f32], coords_y: &[f32]) {
        if !coords_x.is_empty() {
            safe_write_buffer(
                queue,
                &self.coord_x_buffer,
                coords_x,
                "Coastline3DRenderer::update_coords_x",
            );
        }
        if !coords_y.is_empty() {
            safe_write_buffer(
                queue,
                &self.coord_y_buffer,
                coords_y,
                "Coastline3DRenderer::update_coords_y",
            );
        }
    }

    pub fn swap_vertices(&self, device: &wgpu::Device, queue: &wgpu::Queue, vertices: &[f32]) {
        if vertices.is_empty() {
            return;
        }
        let needed = std::mem::size_of_val(vertices) as u64;
        let capacity = self
            .gpu
            .read()
            .map(|resources| resources.coastline_buffer.size())
            .unwrap_or(0);
        if needed <= capacity {
            if let Ok(resources) = self.gpu.read() {
                queue.write_buffer(
                    &resources.coastline_buffer,
                    0,
                    bytemuck::cast_slice(vertices),
                );
            }
        } else {
            let coastline_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("3D Coastline Vertex Buffer (Resized)"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let bind_group = make_bind_group(
                device,
                &self.bind_group_layout,
                &self.uniform_buffer,
                &self.data_buffer,
                &self.coord_x_buffer,
                &self.coord_y_buffer,
                &coastline_buffer,
            );
            if let Ok(mut resources) = self.gpu.write() {
                resources.coastline_buffer = coastline_buffer;
                resources.bind_group = bind_group;
            }
        }
        self.vertex_count
            .store((vertices.len() / 2) as u32, Ordering::Relaxed);
    }
}

#[derive(Copy, Clone)]
pub struct Coastline3DParams {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub plot_kind: u32,
    pub plot_mode: u32,
    pub coord_mode: u32,
    pub lon_bounds: [f32; 2],
    pub lat_bounds: [f32; 2],
    pub color: [f32; 4],
    pub color_range: [f32; 2],
}

pub struct Coastline3DCallback {
    pub renderer: Arc<Coastline3DRenderer>,
    pub params: Coastline3DParams,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for Coastline3DCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let params = self.params;
        let uniforms = Coastline3DUniforms {
            rotation_y: params.rotation_y,
            rotation_x: params.rotation_x,
            aspect_ratio: params.aspect_ratio.max(0.1),
            zoom: params.zoom,
            displacement_strength: params.displacement_strength,
            plot_kind: params.plot_kind,
            plot_mode: params.plot_mode,
            width: self.renderer.width,
            height: self.renderer.height,
            coord_mode: params.coord_mode,
            _pad_bounds: [0; 2],
            lon_bounds: params.lon_bounds,
            lat_bounds: params.lat_bounds,
            color: params.color,
            _pad_color: [0; 2],
            color_range: params.color_range,
            _pad: [0; 2],
            _pad_tail: [0; 2],
        };
        queue.write_buffer(
            &self.renderer.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
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
        let Ok(resources) = self.renderer.gpu.read() else {
            return;
        };
        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &resources.bind_group, &[]);
        rpass.draw(0..vertex_count, 0..1);
    }
}

fn buffer_layout(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BufferBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn make_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    data_buffer: &wgpu::Buffer,
    coord_x_buffer: &wgpu::Buffer,
    coord_y_buffer: &wgpu::Buffer,
    coastline_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("3D Coastline Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: coord_x_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: coord_y_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: coastline_buffer.as_entire_binding(),
            },
        ],
    })
}
