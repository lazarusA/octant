use super::mesh::{Mesh3DCallback, Mesh3DRenderer};

pub type SphereRenderer = Mesh3DRenderer;
pub type SphereCallback = Mesh3DCallback;

impl SphereRenderer {
    pub fn new_sphere(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
    ) -> Self {
        Self::new_sphere_with_coords(
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
    pub fn new_sphere_with_coords(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        matrix_data: &[f32],
        width: usize,
        height: usize,
        coord_x: Option<&[f32]>,
        coord_y: Option<&[f32]>,
    ) -> Self {
        let shader_source =
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/sphere.wgsl"));
        Self::new_with_coords(
            device,
            target_format,
            shader_source,
            Some(wgpu::Face::Back),
            matrix_data,
            width,
            height,
            coord_x,
            coord_y,
        )
    }
}
