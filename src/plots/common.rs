use bytemuck::{Pod, Zeroable};
use eframe::egui;
use wgpu::util::DeviceExt;

/// Maximum buffer size for GPU storage buffers in bytes (128 MiB default WebGPU/Metal limit).
pub const MAX_GPU_STORAGE_BUFFER_BYTES: usize = 128 * 1024 * 1024;
/// Maximum number of f32 elements that fit in a single GPU storage buffer.
pub const MAX_GPU_STORAGE_BUFFER_ELEMENTS: usize =
    MAX_GPU_STORAGE_BUFFER_BYTES / std::mem::size_of::<f32>();

/// Maximum buffer size for GPU vertex / index buffers in bytes (256 MiB default wgpu limit).
pub const MAX_GPU_BUFFER_BYTES: usize = 256 * 1024 * 1024;
/// Maximum number of 2D cells that fit in 3D Surface / Sphere GPU storage buffer (33.5M cells / 128 MiB).
pub const MAX_2D_SURFACE_ELEMENTS: usize = MAX_GPU_STORAGE_BUFFER_ELEMENTS;

/// Standard GPU color, clipping, and range uniforms struct shared across all plots.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PlotColorParams {
    pub colormap: u32,
    pub cmin: f32,
    pub cmax: f32,
    pub use_nan_color: u32,
    pub use_lowclip: u32,
    pub use_highclip: u32,
    pub scale_type: u32,
    pub scale_param: f32,
    pub is_categorical: u32,
    pub num_categories: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub nan_color: [f32; 4],
    pub lowclip_color: [f32; 4],
    pub highclip_color: [f32; 4],
}

impl Default for PlotColorParams {
    fn default() -> Self {
        Self {
            colormap: 0,
            cmin: 0.0,
            cmax: 100.0,
            use_nan_color: 0,
            use_lowclip: 0,
            use_highclip: 0,
            scale_type: 0,
            scale_param: 1.0,
            is_categorical: 0,
            num_categories: 10,
            _pad0: 0,
            _pad1: 0,
            nan_color: [0.0, 0.0, 0.0, 0.0],
            lowclip_color: [0.0, 0.0, 1.0, 1.0],
            highclip_color: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Standard 3D mesh vertex shared across Sphere and Surface heightfield renderers.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshVertex3D {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl MeshVertex3D {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
        2 => Float32x3,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Standard 3D uniform buffer layout for heightfields (Sphere & Surface).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Mesh3DUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub mode: u32,
    pub width: u32,
    pub height: u32,
    pub color: PlotColorParams,
}

/// Parameter bundle for configuring 3D heightfield mesh renderers.
#[derive(Clone, Debug)]
pub struct Mesh3DUniformParams {
    pub color: PlotColorParams,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub mode: u32,
}

/// Standard trait implemented by all Octant WGPU plot renderers.
pub trait PlotRenderer: Send + Sync {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]);
}

/// Setup viewport and scissor rect on a wgpu RenderPass based on egui Rect and PaintCallbackInfo,
/// guaranteeing that the scissor rect is strictly clamped within render target bounds.
/// Returns false if the render target is empty (e.g. window minimized or asleep), in which case
/// the caller should skip drawing to avoid GPU validation errors.
#[inline]
pub fn setup_viewport_and_scissor(
    rpass: &mut wgpu::RenderPass<'static>,
    rect: &egui::Rect,
    info: &eframe::egui::PaintCallbackInfo,
) -> bool {
    let target_rect = info.viewport_in_pixels();
    if target_rect.width_px == 0
        || target_rect.height_px == 0
        || rect.width() <= 0.0
        || rect.height() <= 0.0
    {
        return false;
    }

    let ppp = info.pixels_per_point;
    let max_target_w = target_rect.width_px as f32;
    let max_target_h = target_rect.height_px as f32;

    let vp_x = (rect.min.x * ppp).round().max(0.0);
    let vp_y = (rect.min.y * ppp).round().max(0.0);
    let vp_w = (rect.width() * ppp).round().clamp(1.0, max_target_w);
    let vp_h = (rect.height() * ppp).round().clamp(1.0, max_target_h);

    rpass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);

    // Scissor Rect MUST be strictly contained within target render surface bounds
    let clip_rect = info.clip_rect_in_pixels();

    let target_max_x = target_rect.left_px + target_rect.width_px;
    let target_max_y = target_rect.top_px + target_rect.height_px;

    let clip_max_x = clip_rect.left_px + clip_rect.width_px;
    let clip_max_y = clip_rect.top_px + clip_rect.height_px;

    let sc_min_x = clip_rect
        .left_px
        .clamp(target_rect.left_px, target_max_x)
        .max(0) as u32;
    let sc_min_y = clip_rect
        .top_px
        .clamp(target_rect.top_px, target_max_y)
        .max(0) as u32;
    let sc_max_x = clip_max_x.clamp(target_rect.left_px, target_max_x).max(0) as u32;
    let sc_max_y = clip_max_y.clamp(target_rect.top_px, target_max_y).max(0) as u32;

    let sc_w = sc_max_x
        .saturating_sub(sc_min_x)
        .clamp(1, target_rect.width_px.max(1) as u32);
    let sc_h = sc_max_y
        .saturating_sub(sc_min_y)
        .clamp(1, target_rect.height_px.max(1) as u32);

    rpass.set_scissor_rect(sc_min_x, sc_min_y, sc_w, sc_h);
    true
}

/// Computes viewport aspect ratio from an egui Rect with zero division protection.
#[inline]
pub fn compute_aspect_ratio(rect: &egui::Rect) -> f32 {
    rect.width() / rect.height().max(1.0)
}

/// Helper to create a GPU uniform buffer populated with data.
pub fn create_uniform_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &T,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

/// Helper to create a GPU storage buffer populated with slice data.
pub fn create_storage_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

/// Safely writes slice data to a GPU buffer, checking that the byte size fits within the destination buffer capacity.
/// Returns `true` if the write succeeded, or `false` (with a warning log) if it would overrun.
pub fn safe_write_buffer<T: bytemuck::Pod>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    data: &[T],
    label: &str,
) -> bool {
    let copy_bytes = std::mem::size_of_val(data) as u64;
    if copy_bytes <= buffer.size() {
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
        true
    } else {
        log::warn!(
            "{label}: data size ({copy_bytes} bytes) exceeds GPU buffer capacity ({} bytes), skipping write",
            buffer.size()
        );
        false
    }
}

/// Creates a standard plot bind group layout with binding 0 (Uniform) and binding 1 (Storage).
pub fn create_uniform_storage_bind_group_layout(
    device: &wgpu::Device,
    label: &str,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Creates a standard plot bind group matching the layout with uniform and storage buffers.
pub fn create_uniform_storage_bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    storage_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer.as_entire_binding(),
            },
        ],
    })
}

pub use crate::utils::math::calculate_3d_depth;

/// Helper to create a standard DepthStencilState with Depth32Float format.
pub fn default_depth_stencil_state(
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: Some(depth_write_enabled),
        depth_compare: Some(depth_compare),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

/// Reusable unit cube mesh generator (24 vertices, 36 indices) for instanced 3D rendering.
pub fn build_unit_cube_mesh<V, F>(mut make_vertex: F) -> (Vec<V>, Vec<u32>)
where
    F: FnMut([f32; 3], [f32; 2], [f32; 3]) -> V,
{
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    let mut push_face = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], norm: [f32; 3]| {
        let base_idx = vertices.len() as u32;
        vertices.push(make_vertex(p0, [0.0, 0.0], norm));
        vertices.push(make_vertex(p1, [1.0, 0.0], norm));
        vertices.push(make_vertex(p2, [0.0, 1.0], norm));
        vertices.push(make_vertex(p3, [1.0, 1.0], norm));

        // Counter-Clockwise (CCW) face triangles
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);

        indices.push(base_idx + 2);
        indices.push(base_idx + 1);
        indices.push(base_idx + 3);
    };

    let norm_top = [0.0, 1.0, 0.0];
    let norm_bottom = [0.0, -1.0, 0.0];
    let norm_front = [0.0, 0.0, -1.0];
    let norm_back = [0.0, 0.0, 1.0];
    let norm_left = [-1.0, 0.0, 0.0];
    let norm_right = [1.0, 0.0, 0.0];

    // 1. Top Face (+Y normal in world space: z=1.0)
    push_face(
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        norm_top,
    );
    // 2. Bottom Base (-Y normal in world space: z=0.0)
    push_face(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        norm_bottom,
    );
    // 3. Front Wall (-Z normal in world space: y=0.0)
    push_face(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        norm_front,
    );
    // 4. Back Wall (+Z normal in world space: y=1.0)
    push_face(
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        norm_back,
    );
    // 5. Left Wall (-X normal in world space: x=0.0)
    push_face(
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        norm_left,
    );
    // 6. Right Wall (+X normal in world space: x=1.0)
    push_face(
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        norm_right,
    );

    (vertices, indices)
}

/// Reusable unit quad mesh generator (4 vertices, 6 indices) for instanced 2D/3D quad grid rendering.
pub fn build_unit_quad_mesh<V, F>(mut make_vertex: F) -> (Vec<V>, Vec<u32>)
where
    F: FnMut([f32; 3], [f32; 2], [f32; 3]) -> V,
{
    let norm = [0.0, 1.0, 0.0];
    let vertices = vec![
        make_vertex([0.0, 0.0, 0.0], [0.0, 0.0], norm),
        make_vertex([1.0, 0.0, 0.0], [1.0, 0.0], norm),
        make_vertex([0.0, 1.0, 0.0], [0.0, 1.0], norm),
        make_vertex([1.0, 1.0, 0.0], [1.0, 1.0], norm),
    ];
    let indices = vec![0, 2, 1, 1, 2, 3];
    (vertices, indices)
}
