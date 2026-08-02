/// 3D Surface Plot with Height Displacement.
///
/// Displaces vertices along the Z-axis based on matrix scalar scalar values
/// to render interactive 3D terrain/elevation surfaces.
pub struct SurfacePlot {
    pub displacement_scale: f32,
}

impl SurfacePlot {
    pub fn new(displacement_scale: f32) -> Self {
        Self { displacement_scale }
    }
}
