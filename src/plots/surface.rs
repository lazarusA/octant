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
        Self::new_surface_with_coords(
            device,
            target_format,
            matrix_data,
            width,
            height,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_surface_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        let shader_source =
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/surface.wgsl"));
        Self::new_with_coords(
            device,
            target_format,
            shader_source,
            None,
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }
}
