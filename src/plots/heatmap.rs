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
    pub coord_mode: u32,
    pub aspect_scale: [f32; 2],
    pub width: u32,
    pub height: u32,
    pub tile_bounds: [f32; 4],
    pub lut_size_x: u32,
    pub lut_size_y: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub color: super::common::PlotColorParams,
}

use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct HeatmapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    coord_x_buffer: wgpu::Buffer,
    coord_y_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
    width: AtomicU32,
    height: AtomicU32,
    coord_mode: AtomicU32,
    lut_size_x: AtomicU32,
    lut_size_y: AtomicU32,
    tile_bounds: RwLock<[f32; 4]>,
}

impl HeatmapRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        Self::new_with_coords(
            device,
            target_format,
            matrix_data,
            width,
            height,
            None,
            None,
        )
    }

    pub fn new_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        let shader_source =
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/heatmap.wgsl"));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Heatmap Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let initial_coord_mode = if coord_x.is_some() || coord_y.is_some() {
            2
        } else {
            0
        };

        let lut_size_x = if initial_coord_mode == 2 {
            crate::data::coordinates::compute_coord_lut_size(width)
        } else {
            0
        };
        let lut_size_y = if initial_coord_mode == 2 {
            crate::data::coordinates::compute_coord_lut_size(height)
        } else {
            0
        };

        let initial_uniforms = HeatmapUniforms {
            pan: [0.0, 0.0],
            zoom: 1.0,
            coord_mode: initial_coord_mode,
            aspect_scale: [1.0, 1.0],
            width: width.max(1) as u32,
            height: height.max(1) as u32,
            tile_bounds: [0.0, 0.0, 1.0, 1.0],
            lut_size_x: lut_size_x as u32,
            lut_size_y: lut_size_y as u32,
            _pad0: 0,
            _pad1: 0,
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

        let mut padded_coords_x = if let Some(cx) = coord_x.filter(|s| !s.is_empty()) {
            crate::data::coordinates::build_1d_coord_lut(cx, lut_size_x)
        } else {
            vec![0.0; 4]
        };
        let min_coord_x_capacity = lut_size_x.max(4096);
        if padded_coords_x.len() < min_coord_x_capacity {
            padded_coords_x.resize(min_coord_x_capacity, 0.0);
        }

        let mut padded_coords_y = if let Some(cy) = coord_y.filter(|s| !s.is_empty()) {
            crate::data::coordinates::build_1d_coord_lut(cy, lut_size_y)
        } else {
            vec![0.0; 4]
        };
        let min_coord_y_capacity = lut_size_y.max(4096);
        if padded_coords_y.len() < min_coord_y_capacity {
            padded_coords_y.resize(min_coord_y_capacity, 0.0);
        }

        let coord_x_buffer = super::common::create_storage_buffer(
            device,
            "Heatmap Coord X Storage Buffer",
            &padded_coords_x,
        );

        let coord_y_buffer = super::common::create_storage_buffer(
            device,
            "Heatmap Coord Y Storage Buffer",
            &padded_coords_y,
        );

        let bind_group_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Heatmap Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );

        let bind_group = super::common::create_plot_with_coords_bind_group(
            device,
            "Heatmap Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
            &coord_x_buffer,
            &coord_y_buffer,
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
            coord_x_buffer,
            coord_y_buffer,
            uniform_buffer,
            bind_group,
            num_indices: indices.len() as u32,
            width: AtomicU32::new(width as u32),
            height: AtomicU32::new(height as u32),
            coord_mode: AtomicU32::new(initial_coord_mode),
            lut_size_x: AtomicU32::new(lut_size_x as u32),
            lut_size_y: AtomicU32::new(lut_size_y as u32),
            tile_bounds: RwLock::new([0.0, 0.0, 1.0, 1.0]),
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        color: &super::common::PlotColorParams,
        pan: [f32; 2],
        zoom: f32,
        aspect_scale: [f32; 2],
        coord_mode: u32,
    ) {
        let tile_bounds = self
            .tile_bounds
            .read()
            .map(|b| *b)
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);
        self.coord_mode.store(coord_mode, Ordering::Relaxed);
        let uniforms = HeatmapUniforms {
            pan,
            zoom,
            coord_mode,
            aspect_scale,
            width: self.width.load(Ordering::Relaxed),
            height: self.height.load(Ordering::Relaxed),
            tile_bounds,
            lut_size_x: self.lut_size_x.load(Ordering::Relaxed),
            lut_size_y: self.lut_size_y.load(Ordering::Relaxed),
            _pad0: 0,
            _pad1: 0,
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_colormap(&self, queue: &wgpu::Queue, colormap: u32) {
        let color = super::common::PlotColorParams {
            colormap,
            ..Default::default()
        };
        self.update_uniforms(
            queue,
            &color,
            [0.0, 0.0],
            1.0,
            [1.0, 1.0],
            self.coord_mode.load(Ordering::Relaxed),
        );
    }

    /// Fast GPU Storage Buffer data channel upload
    pub fn update_data(&self, queue: &wgpu::Queue, matrix_data: &[f32]) {
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "HeatmapRenderer::update_data",
        );
    }

    pub fn update_coords(&self, queue: &wgpu::Queue, coords_x: &[f32], coords_y: &[f32]) {
        if !coords_x.is_empty() {
            let w = self.width.load(Ordering::Relaxed) as usize;
            let lut_size_x = crate::data::coordinates::compute_coord_lut_size(w);
            self.lut_size_x.store(lut_size_x as u32, Ordering::Relaxed);
            let lut_x = crate::data::coordinates::build_1d_coord_lut(coords_x, lut_size_x);
            super::common::safe_write_buffer(
                queue,
                &self.coord_x_buffer,
                &lut_x,
                "HeatmapRenderer::update_coords_x",
            );
        }
        if !coords_y.is_empty() {
            let h = self.height.load(Ordering::Relaxed) as usize;
            let lut_size_y = crate::data::coordinates::compute_coord_lut_size(h);
            self.lut_size_y.store(lut_size_y as u32, Ordering::Relaxed);
            let lut_y = crate::data::coordinates::build_1d_coord_lut(coords_y, lut_size_y);
            super::common::safe_write_buffer(
                queue,
                &self.coord_y_buffer,
                &lut_y,
                "HeatmapRenderer::update_coords_y",
            );
        }
    }

    /// Updates data, dimensions, and tile bounds for dynamic viewport LOD resampling
    pub fn update_data_and_dimensions(
        &self,
        queue: &wgpu::Queue,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        tile_bounds: [f32; 4],
    ) {
        self.width.store(width as u32, Ordering::Relaxed);
        self.height.store(height as u32, Ordering::Relaxed);
        if let Ok(mut b) = self.tile_bounds.write() {
            *b = tile_bounds;
        }
        super::common::safe_write_buffer(
            queue,
            &self.data_buffer,
            matrix_data,
            "HeatmapRenderer::update_data_and_dimensions",
        );
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

impl super::traits::PlotRenderer for HeatmapRenderer {
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
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        data: &crate::data::RenderData,
        _params: &super::traits::PlotRenderParams,
    ) -> Option<super::traits::HoverSample> {
        let crate::data::RenderData::Matrix(m) = data else {
            return None;
        };
        if !rect.contains(pointer_pos) || m.width == 0 || m.height == 0 {
            return None;
        }
        let nx = ((pointer_pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 1.0);
        let ny = ((pointer_pos.y - rect.min.y) / rect.height().max(1.0)).clamp(0.0, 1.0);
        let (px, py) = m.grid.find_cell_from_norm(nx, ny, m.width, m.height);
        let val = m.values.get(py * m.width + px).copied().unwrap_or(f32::NAN);
        let (cell_lon, cell_lat) = m.grid.cell_center_lon_lat_rad(px, py, m.width, m.height);
        Some(super::traits::HoverSample {
            cell_x: px,
            cell_y: py,
            value: val,
            coord_lon_lat: Some((cell_lon.to_degrees() as f64, cell_lat.to_degrees() as f64)),
            world_pos: None,
        })
    }
}

pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub coord_mode: u32,
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
            self.coord_mode,
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
