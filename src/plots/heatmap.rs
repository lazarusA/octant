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

/// The heatmap shader: the shared data quad (`common/quad.wgsl`) colored
/// through a colormap.
pub(crate) const HEATMAP_WGSL: &str = crate::assemble_plot_with_coords_shader!(concat!(
    include_str!("shaders/common/quad.wgsl"),
    "\n",
    include_str!("shaders/heatmap.wgsl")
));

/// A coordinate lookup table's storage buffer, room for `lut_size` entries
/// (at least 4096) and holding `coords`' table when given.
fn coord_lut_buffer(
    device: &wgpu::Device,
    label: &str,
    coords: Option<&[f32]>,
    lut_size: usize,
) -> wgpu::Buffer {
    let mut lut = match coords.filter(|c| !c.is_empty()) {
        Some(c) => crate::data::coordinates::build_1d_coord_lut(c, lut_size),
        None => Vec::new(),
    };
    lut.resize(lut.len().max(lut_size.max(4096)), 0.0);
    super::common::create_storage_buffer(device, label, &lut)
}

/// A pipeline drawing the data quad (`HeatmapVertex`es) with `source`'s
/// `vs_main` and `fs_main`, alpha blended, without depth testing.
pub(crate) fn quad_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    name: &str,
    source: &str,
    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(name),
        bind_group_layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(&layout),
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
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(super::common::default_depth_stencil_state(
            false,
            wgpu::CompareFunction::Always,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// The data quad, uv (0, 0) at the top left.
const QUAD_VERTICES: [HeatmapVertex; 4] = [
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
const QUAD_INDICES: [u32; 6] = [0, 2, 1, 1, 2, 3];

pub struct HeatmapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    data_buffer: wgpu::Buffer,
    coord_x_buffer: wgpu::Buffer,
    coord_y_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: AtomicU32,
    height: AtomicU32,
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
        let has_coords = coord_x.is_some() || coord_y.is_some();
        let lut_size = |n| {
            if has_coords {
                crate::data::coordinates::compute_coord_lut_size(n)
            } else {
                0
            }
        };
        let (lut_size_x, lut_size_y) = (lut_size(width), lut_size(height));
        let uniforms = HeatmapUniforms {
            pan: [0.0, 0.0],
            zoom: 1.0,
            coord_mode: if has_coords { 2 } else { 0 },
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
        let uniform_buffer =
            super::common::create_uniform_buffer(device, "Heatmap Uniform Buffer", &uniforms);
        let capacity = (width * height).clamp(
            2048 * 2048,
            crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS,
        );
        let mut padded = matrix_data.to_vec();
        padded.resize(padded.len().max(capacity), 0.0);
        let data_buffer =
            super::common::create_storage_buffer(device, "Heatmap Data Storage Buffer", &padded);
        let coord_x_buffer = coord_lut_buffer(device, "Heatmap Coord X", coord_x, lut_size_x);
        let coord_y_buffer = coord_lut_buffer(device, "Heatmap Coord Y", coord_y, lut_size_y);
        let layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Heatmap Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );
        let bind_group = super::common::create_plot_with_coords_bind_group(
            device,
            "Heatmap Bind Group",
            &layout,
            &uniform_buffer,
            &data_buffer,
            &coord_x_buffer,
            &coord_y_buffer,
        );
        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let layouts = [Some(&layout), Some(&colormap_layout)];
        let buffer = |label, contents: &[u8], usage| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };
        Self {
            render_pipeline: quad_pipeline(
                device,
                target_format,
                "Heatmap",
                HEATMAP_WGSL,
                &layouts,
            ),
            vertex_buffer: buffer(
                "Heatmap Vertex Buffer",
                bytemuck::cast_slice(&QUAD_VERTICES),
                wgpu::BufferUsages::VERTEX,
            ),
            index_buffer: buffer(
                "Heatmap Index Buffer",
                bytemuck::cast_slice(&QUAD_INDICES),
                wgpu::BufferUsages::INDEX,
            ),
            data_buffer,
            coord_x_buffer,
            coord_y_buffer,
            uniform_buffer,
            bind_group,
            width: AtomicU32::new(width as u32),
            height: AtomicU32::new(height as u32),
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

    /// Binds the data (group 0) and the quad, for pipelines that share them
    /// (label images, `labels.rs`).
    pub(crate) fn bind_quad(&self, rpass: &mut wgpu::RenderPass<'static>) {
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    /// Draws the quad bound by `bind_quad`.
    pub(crate) fn draw_quad(&self, rpass: &mut wgpu::RenderPass<'static>) {
        rpass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
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

/// Paints a heatmap renderer's data: through its colormap, or for a label
/// layer (`labels`) in its label colors.
pub struct HeatmapCallback {
    pub renderer: Arc<HeatmapRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub coord_mode: u32,
    pub labels: Option<super::labels::LabelDraw>,
}

impl eframe::egui_wgpu::CallbackTrait for HeatmapCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        super::colormap_atlas::prepare(device, queue, callback_resources);
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.pan,
            self.zoom,
            self.aspect_scale,
            self.coord_mode,
        );
        if let Some(labels) = &self.labels {
            labels
                .renderer
                .update_params(queue, labels.outline, labels.opacity);
        }
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
        if let Some(labels) = &self.labels {
            labels.renderer.draw(rpass, &self.renderer);
            return;
        }
        if !super::colormap_atlas::bind(rpass, callback_resources) {
            return;
        }

        rpass.set_pipeline(&self.renderer.render_pipeline);
        self.renderer.bind_quad(rpass);
        self.renderer.draw_quad(rpass);
    }
}
