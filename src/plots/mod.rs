pub mod common;
pub mod heatmap;
pub mod line;
pub mod point_cloud;
pub mod sphere;
pub mod surface;
pub mod volume;

pub use common::PlotRenderer;
pub use heatmap::{HeatmapCallback, HeatmapRenderer, MatrixCallback, MatrixRenderer};
pub use line::{LineCallback, LineRenderer};
pub use point_cloud::{PointCloudCallback, PointCloudRenderer};
pub use sphere::{SphereCallback, SpherePlot, SphereRenderer};
pub use surface::{SurfaceCallback, SurfacePlot, SurfaceRenderer};
pub use volume::{VolumeCallback, VolumeRenderer};

/// Supported visualization plot types in Octant Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlotType {
    #[default]
    Heatmap,
    Line,
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
            PlotType::Line => "1D Line Chart",
            PlotType::Surface => "3D Surface / Blocks",
            PlotType::Block => "3D Voxel / Block",
            PlotType::Volume => "3D Volume Raycasting",
            PlotType::Sphere => "3D Globe Projection",
            PlotType::PointCloud => "3D Point Cloud",
        }
    }
}

/// Assembles a plot WGSL shader by prepending all shared WGSL colormap modules.
#[macro_export]
macro_rules! assemble_plot_shader {
    ($plot_shader:expr) => {
        concat!(
            include_str!("shaders/colormaps/viridis.wgsl"),
            "\n",
            include_str!("shaders/colormaps/plasma.wgsl"),
            "\n",
            include_str!("shaders/colormaps/inferno.wgsl"),
            "\n",
            include_str!("shaders/colormaps/magma.wgsl"),
            "\n",
            include_str!("shaders/colormaps/turbo.wgsl"),
            "\n",
            include_str!("shaders/colormaps/coolwarm.wgsl"),
            "\n",
            include_str!("shaders/colormaps/cividis.wgsl"),
            "\n",
            include_str!("shaders/colormaps/mod.wgsl"),
            "\n",
            $plot_shader
        )
    };
}
