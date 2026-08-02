pub mod heatmap;
pub mod surface;
pub mod block;
pub mod volume;
pub mod sphere;
pub mod point_cloud;

pub use heatmap::{HeatmapCallback, HeatmapRenderer, MatrixCallback, MatrixRenderer};
pub use surface::{SurfaceCallback, SurfaceRenderer, SurfacePlot};
pub use block::BlockPlot;
pub use volume::VolumePlot; 
pub use sphere::{SphereCallback, SphereRenderer, SpherePlot};

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
            include_str!("shaders/colormaps/viridis.wgsl"), "\n",
            include_str!("shaders/colormaps/plasma.wgsl"), "\n",
            include_str!("shaders/colormaps/inferno.wgsl"), "\n",
            include_str!("shaders/colormaps/magma.wgsl"), "\n",
            include_str!("shaders/colormaps/turbo.wgsl"), "\n",
            include_str!("shaders/colormaps/coolwarm.wgsl"), "\n",
            include_str!("shaders/colormaps/cividis.wgsl"), "\n",
            include_str!("shaders/colormaps/mod.wgsl"), "\n",
            $plot_shader
        )
    };
}
