use super::mesh::{Mesh3DCallback, Mesh3DRenderer};

pub type SurfaceRenderer = Mesh3DRenderer;
pub type SurfaceCallback = Mesh3DCallback;

impl SurfaceRenderer {
    pub fn new_surface(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/surface.wgsl"));
        Self::new(
            device,
            target_format,
            shader_source,
            None,
            matrix_data,
            width,
            height,
        )
    }
}
