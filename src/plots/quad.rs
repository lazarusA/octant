//! The data quad shared by the heatmap and label images: its vertices, its
//! render pipeline, and its data and coordinate lookup table storage buffers,
//! sized past their initial contents so later data fits in place.

use bytemuck::{Pod, Zeroable};

/// The data quad's uniforms (`QuadUniforms` of `shaders/common/quad.wgsl`),
/// shared by the heatmap and label pipelines.
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

/// Tile bounds covering the whole data.
pub(crate) const FULL_TILE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

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

/// The data quad, uv (0, 0) at the top left.
pub(crate) const QUAD_VERTICES: [HeatmapVertex; 4] = [
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
pub(crate) const QUAD_INDICES: [u32; 6] = [0, 2, 1, 1, 2, 3];

/// The data quad's vertex and index buffers, labeled after `name`.
pub(crate) fn quad_buffers(device: &wgpu::Device, name: &str) -> (wgpu::Buffer, wgpu::Buffer) {
    use wgpu::util::DeviceExt;
    let buffer = |kind: &str, contents: &[u8], usage| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{name} {kind} Buffer")),
            contents,
            usage,
        })
    };
    (
        buffer(
            "Vertex",
            bytemuck::cast_slice(&QUAD_VERTICES),
            wgpu::BufferUsages::VERTEX,
        ),
        buffer(
            "Index",
            bytemuck::cast_slice(&QUAD_INDICES),
            wgpu::BufferUsages::INDEX,
        ),
    )
}

/// A `STORAGE | COPY_DST` buffer of at least `min_len` values, holding
/// `data` and zeros after it, written while mapped at creation so no padded
/// copy of `data` is built on the CPU.
fn storage_buffer_at_least(
    device: &wgpu::Device,
    label: &str,
    data: &[f32],
    min_len: usize,
) -> wgpu::Buffer {
    let len = data.len().max(min_len).max(1);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: len.saturating_mul(size_of::<f32>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });
    let bytes: &[u8] = bytemuck::cast_slice(data);
    if !bytes.is_empty()
        && let Ok(mut view) = buffer
            .slice(..bytes.len() as wgpu::BufferAddress)
            .get_mapped_range_mut()
    {
        view.copy_from_slice(bytes);
    }
    buffer.unmap();
    buffer
}

/// A coordinate lookup table of `coords` for an axis of `len` cells, with
/// its size.
pub(crate) fn coord_lut(coords: &[f32], len: usize) -> (usize, Vec<f32>) {
    let size = crate::data::coordinates::compute_coord_lut_size(len);
    (
        size,
        crate::data::coordinates::build_1d_coord_lut(coords, size),
    )
}

/// An axis's coordinate lookup table buffer, room for at least 4096 entries,
/// holding `coords`' table when given (`coord_lut` over `len` cells), with
/// the table's size (0 without coordinates on either axis, `has_coords`).
pub(crate) fn coord_lut_buffer(
    device: &wgpu::Device,
    label: &str,
    coords: Option<&[f32]>,
    len: usize,
    has_coords: bool,
) -> (wgpu::Buffer, u32) {
    let size = if has_coords {
        crate::data::coordinates::compute_coord_lut_size(len)
    } else {
        0
    };
    let lut = match coords.filter(|c| !c.is_empty()) {
        Some(c) => coord_lut(c, len).1,
        None => Vec::new(),
    };
    let buffer = storage_buffer_at_least(device, label, &lut, size.max(4096));
    (buffer, size as u32)
}

/// The data's storage buffer, room for `cells` values (at least 2048^2, at
/// most the storage limit unless the data is larger) so smaller data later
/// fits in place.
pub(crate) fn data_buffer(device: &wgpu::Device, data: &[f32], cells: usize) -> wgpu::Buffer {
    let capacity = cells.clamp(2048 * 2048, super::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS);
    storage_buffer_at_least(device, "Heatmap Data Storage Buffer", data, capacity)
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
