pub mod common;
pub mod heatmap;
pub mod line;
pub mod mesh;
pub mod point_cloud;
pub mod sphere;
pub mod surface;
pub mod volume;

pub use common::{
    Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D, PlotColorParams, PlotRenderer,
};
pub use heatmap::{HeatmapCallback, HeatmapRenderer, MatrixCallback, MatrixRenderer};
pub use line::{LineCallback, LineRenderer};
pub use mesh::{Mesh3DCallback, Mesh3DRenderer};
pub use point_cloud::{PointCloudCallback, PointCloudRenderer, PointCloudUniformParams};
pub use sphere::{SphereCallback, SphereRenderer};
pub use surface::{SurfaceCallback, SurfaceRenderer};
pub use volume::{VolumeCallback, VolumeRenderer, VolumeUniformParams};

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_plot_shaders_parse_cleanly() {
        let shaders = [
            (
                "sphere",
                crate::assemble_plot_shader!(include_str!("shaders/sphere.wgsl")),
            ),
            (
                "surface",
                crate::assemble_plot_shader!(include_str!("shaders/surface.wgsl")),
            ),
            (
                "heatmap",
                crate::assemble_plot_shader!(include_str!("shaders/heatmap.wgsl")),
            ),
            (
                "volume",
                crate::assemble_plot_shader!(include_str!("shaders/volume.wgsl")),
            ),
            (
                "line",
                crate::assemble_plot_shader!(include_str!("shaders/line.wgsl")),
            ),
            (
                "point_cloud",
                crate::assemble_plot_shader!(include_str!("shaders/point_cloud.wgsl")),
            ),
        ];

        for (name, source) in shaders {
            let mut validator = wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            );
            let module = wgpu::naga::front::wgsl::parse_str(source)
                .unwrap_or_else(|e| panic!("Failed to parse WGSL shader '{name}': {e}"));
            validator
                .validate(&module)
                .unwrap_or_else(|e| panic!("Failed to validate WGSL shader '{name}': {e}"));
        }
    }
}
