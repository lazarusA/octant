use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointCloudVertex {
    pub position: [f32; 2],
}

impl PointCloudVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointCloudUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub color: super::common::PlotColorParams,
}

#[derive(Copy, Clone, Debug)]
pub struct PointCloudUniformParams {
    pub color: super::common::PlotColorParams,
    pub rot_y: f32,
    pub rot_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
}
