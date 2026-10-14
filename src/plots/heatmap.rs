use super::quad::{
    QUAD_INDICES, coord_lut, coord_lut_buffer, data_buffer, quad_buffers, quad_pipeline,
};
use bytemuck::{Pod, Zeroable};

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

/// Tile bounds covering the whole data.
const FULL_TILE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

impl Default for HeatmapUniforms {
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: 1.0,
            coord_mode: 0,
            aspect_scale: [1.0, 1.0],
            width: 1,
            height: 1,
            tile_bounds: FULL_TILE,
            lut_size_x: 0,
            lut_size_y: 0,
            _pad0: 0,
            _pad1: 0,
            color: super::common::PlotColorParams::default(),
        }
    }
}

/// The heatmap shader: the shared data quad (`common/quad.wgsl`) colored
/// through a colormap.
pub(crate) const HEATMAP_WGSL: &str = crate::assemble_plot_with_coords_shader!(concat!(
    include_str!("shaders/common/quad.wgsl"),
    "\n",
    include_str!("shaders/heatmap.wgsl")
));

/// The heatmap's group 0 (`uniform`, `data`, X and Y coordinate lookup
/// tables, in binding order) and its layout.
fn data_bind_group(
    device: &wgpu::Device,
    [uniform, data, coord_x, coord_y]: [&wgpu::Buffer; 4],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    use super::common as c;
    let layout = c::create_plot_with_coords_bind_group_layout(
        device,
        "Heatmap Bind Group Layout",
        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        wgpu::ShaderStages::FRAGMENT,
    );
    let bind_group = c::create_plot_with_coords_bind_group(
        device,
        "Heatmap Bind Group",
        &layout,
        uniform,
        data,
        coord_x,
        coord_y,
    );
    (layout, bind_group)
}

pub struct HeatmapRenderer {
    pub(super) render_pipeline: wgpu::RenderPipeline,
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
        let (coord_x, lut_size_x) =
            coord_lut_buffer(device, "Heatmap Coord X", coord_x, width, has_coords);
        let (coord_y, lut_size_y) =
            coord_lut_buffer(device, "Heatmap Coord Y", coord_y, height, has_coords);
        let uniforms = HeatmapUniforms {
            coord_mode: if has_coords { 2 } else { 0 },
            width: width.max(1) as u32,
            height: height.max(1) as u32,
            lut_size_x,
            lut_size_y,
            ..HeatmapUniforms::default()
        };
        let uniform_buffer =
            super::common::create_uniform_buffer(device, "Heatmap Uniform Buffer", &uniforms);
        let data_buffer = data_buffer(device, matrix_data, width.saturating_mul(height));
        let buffers = [&uniform_buffer, &data_buffer, &coord_x, &coord_y];
        let (layout, bind_group) = data_bind_group(device, buffers);
        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let layouts = [Some(&layout), Some(&colormap_layout)];
        let (vertex_buffer, index_buffer) = quad_buffers(device, "Heatmap");
        Self {
            render_pipeline: quad_pipeline(
                device,
                target_format,
                "Heatmap",
                HEATMAP_WGSL,
                &layouts,
            ),
            vertex_buffer,
            index_buffer,
            data_buffer,
            coord_x_buffer: coord_x,
            coord_y_buffer: coord_y,
            uniform_buffer,
            bind_group,
            width: AtomicU32::new(width as u32),
            height: AtomicU32::new(height as u32),
            lut_size_x: AtomicU32::new(lut_size_x),
            lut_size_y: AtomicU32::new(lut_size_y),
            tile_bounds: RwLock::new(FULL_TILE),
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
        let tile_bounds = self.tile_bounds.read().map(|b| *b).unwrap_or(FULL_TILE);
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

    /// Rewrites the coordinate lookup tables of the axes given coordinates.
    pub fn update_coords(&self, queue: &wgpu::Queue, coords_x: &[f32], coords_y: &[f32]) {
        let axes = [
            (
                coords_x,
                &self.width,
                &self.lut_size_x,
                &self.coord_x_buffer,
            ),
            (
                coords_y,
                &self.height,
                &self.lut_size_y,
                &self.coord_y_buffer,
            ),
        ];
        for (coords, len, lut_size, buffer) in axes {
            if coords.is_empty() {
                continue;
            }
            let (size, lut) = coord_lut(coords, len.load(Ordering::Relaxed) as usize);
            lut_size.store(size as u32, Ordering::Relaxed);
            super::common::safe_write_buffer(queue, buffer, &lut, "HeatmapRenderer::update_coords");
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
