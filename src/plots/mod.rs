pub mod heatmap;
pub mod surface;
pub mod block;
pub mod volume;
pub mod sphere;
pub mod point_cloud;

pub use heatmap::{HeatmapCallback, HeatmapRenderer, MatrixCallback, MatrixRenderer};
pub use surface::SurfacePlot;
pub use block::BlockPlot;
pub use volume::VolumePlot;
pub use sphere::SpherePlot;
pub use point_cloud::PointCloudPlot;

/// Supported visualization plot types in Octant Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlotType {
    #[default]
    Heatmap,
    Surface,
    Block,
    Volume,
    Sphere,
    PointCloud,
}

impl PlotType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PlotType::Heatmap => "2D Flatmap Heatmap",
            PlotType::Surface => "3D Displaced Surface",
            PlotType::Block => "3D Voxel / Block",
            PlotType::Volume => "3D Volume Raycasting",
            PlotType::Sphere => "3D Globe Projection",
            PlotType::PointCloud => "3D Point Cloud",
        }
    }
}
