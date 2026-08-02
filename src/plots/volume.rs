/// 3D Volumetric Raycasting Plot.
///
/// Samples 3D scalar fields (e.g. atmospheric layers, ocean depths) via GPU raymarching.
pub struct VolumePlot {
    pub step_count: u32,
    pub opacity_scale: f32,
}

impl VolumePlot {
    pub fn new(step_count: u32, opacity_scale: f32) -> Self {
        Self { step_count, opacity_scale }
    }
}
