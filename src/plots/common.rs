use bytemuck::{Pod, Zeroable};
use eframe::egui;
use wgpu::util::DeviceExt;

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

/// Setup viewport and scissor rect on a wgpu RenderPass based on egui Rect and pixels_per_point.
#[inline]
pub fn setup_viewport_and_scissor(
    rpass: &mut wgpu::RenderPass<'static>,
    rect: &egui::Rect,
    pixels_per_point: f32,
) {
    let px_x = (rect.min.x * pixels_per_point).max(0.0) as u32;
    let px_y = (rect.min.y * pixels_per_point).max(0.0) as u32;
    let px_w = (rect.width() * pixels_per_point).max(1.0) as u32;
    let px_h = (rect.height() * pixels_per_point).max(1.0) as u32;

    rpass.set_viewport(px_x as f32, px_y as f32, px_w as f32, px_h as f32, 0.0, 1.0);
    rpass.set_scissor_rect(px_x, px_y, px_w, px_h);
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

/// Infers 3D volume/grid depth dimension from total elements count and 2D grid dimensions.
#[inline]
pub fn calculate_3d_depth(total_len: usize, width: u32, height: u32) -> u32 {
    (total_len as u32 / (width.max(1) * height.max(1))).max(1)
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

        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 1);

        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    };

    let norm_top = [0.0, 1.0, 0.0];
    let norm_bottom = [0.0, -1.0, 0.0];
    let norm_front = [0.0, 0.0, -1.0];
    let norm_back = [0.0, 0.0, 1.0];
    let norm_left = [-1.0, 0.0, 0.0];
    let norm_right = [1.0, 0.0, 0.0];

    // 1. Top Face (z=1.0)
    push_face([0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], norm_top);
    // 2. Bottom Base (z=0.0)
    push_face([0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], norm_bottom);
    // 3. Front Wall (y=0.0)
    push_face([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], norm_front);
    // 4. Back Wall (y=1.0)
    push_face([0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], norm_back);
    // 5. Left Wall (x=0.0)
    push_face([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], norm_left);
    // 6. Right Wall (x=1.0)
    push_face([1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], norm_right);

    (vertices, indices)
}
