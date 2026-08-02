/// 3D Point Cloud Plot.
///
/// Renders sparse or dense multi-dimensional point measurements in 3D coordinate space.
pub struct PointCloudPlot {
    pub point_size: f32,
}

impl PointCloudPlot {
    pub fn new(point_size: f32) -> Self {
        Self { point_size }
    }
}
