pub mod coastline;
pub mod common;
pub mod heatmap;
pub mod line;
pub mod mesh;
pub mod point_cloud;
pub mod sphere;
pub mod surface;
pub mod traits;
pub mod volume;

pub use coastline::{
    Coastline3DCallback, Coastline3DParams, Coastline3DRenderer, CoastlineBuffer,
    CoastlineCallback, CoastlineFetchResult, CoastlineLod, CoastlineReceiver, CoastlineRenderer,
    CoastlineSender, dataset_geo_bounds, expand_coastline_line_list, fetch_coastline_async,
    load_coastline_sync,
};
pub use common::{Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D, PlotColorParams};
pub use heatmap::{HeatmapCallback, HeatmapRenderer, MatrixCallback, MatrixRenderer};
pub use line::{LineCallback, LineRenderer};
pub use mesh::{Mesh3DCallback, Mesh3DRenderer};
pub use point_cloud::{PointCloudCallback, PointCloudRenderer, PointCloudUniformParams};
pub use sphere::{SphereCallback, SphereRenderer};
pub use surface::{SurfaceCallback, SurfaceRenderer};
pub use traits::{HoverSample, PlotRenderParams, PlotRenderer};
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

/// Assembles a base plot WGSL shader by prepending colormaps and shared 3D camera utilities.
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
            include_str!("shaders/common/camera3d.wgsl"),
            "\n",
            include_str!("shaders/common/lighting.wgsl"),
            "\n",
            $plot_shader
        )
    };
}

/// Assembles a 2D/3D plot WGSL shader with 1D coordinate buffer bindings, projections, and coordinate math.
#[macro_export]
macro_rules! assemble_plot_with_coords_shader {
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
            include_str!("shaders/common/coords.wgsl"),
            "\n",
            include_str!("shaders/common/camera3d.wgsl"),
            "\n",
            include_str!("shaders/common/lighting.wgsl"),
            "\n",
            include_str!("shaders/common/projections.wgsl"),
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
                crate::assemble_plot_with_coords_shader!(include_str!("shaders/sphere.wgsl")),
            ),
            (
                "surface",
                crate::assemble_plot_with_coords_shader!(include_str!("shaders/surface.wgsl")),
            ),
            (
                "heatmap",
                crate::assemble_plot_with_coords_shader!(include_str!("shaders/heatmap.wgsl")),
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
            ("coastline", include_str!("shaders/coastline.wgsl")),
            (
                "coastline_3d",
                concat!(
                    include_str!("shaders/common/coords.wgsl"),
                    "\n",
                    include_str!("shaders/common/camera3d.wgsl"),
                    "\n",
                    include_str!("shaders/common/projections.wgsl"),
                    "\n",
                    include_str!("shaders/coastline_3d.wgsl"),
                ),
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
