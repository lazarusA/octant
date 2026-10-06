pub mod coastline;
pub mod colormap_atlas;
pub mod common;
pub mod device;
pub mod fullscreen;
pub mod heatmap;
pub mod line;
pub mod mesh;
mod mesh_draw;
pub mod oit;
pub mod point_cloud;
mod point_cloud_draw;
mod point_cloud_types;
pub mod sphere;
pub mod surface;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) mod test_gpu;
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
pub use volume::{VolumeCallback, VolumeEncoding, VolumeRenderer, VolumeUniformParams};

/// Supported visualization plot types in Octant Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlotType {
    #[default]
    Heatmap,
    Line,
    Surface,
    Volume,
    Sphere,
    PointCloud,
}

impl PlotType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PlotType::Heatmap => "Heatmap",
            PlotType::Line => "1D Line",
            PlotType::Surface => "Surface",
            PlotType::Volume => "Volume",
            PlotType::Sphere => "Sphere",
            PlotType::PointCloud => "Point Cloud",
        }
    }
}

/// Assembles a base plot WGSL shader by prepending the colormap atlas sampler and shared 3D camera utilities.
#[macro_export]
macro_rules! assemble_plot_shader {
    ($plot_shader:expr) => {
        concat!(
            include_str!("shaders/colormaps/mod.wgsl"),
            "\n",
            include_str!("shaders/common/camera3d.wgsl"),
            "\n",
            include_str!("shaders/common/lighting.wgsl"),
            "\n",
            include_str!("shaders/common/oit.wgsl"),
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
            include_str!("shaders/colormaps/mod.wgsl"),
            "\n",
            include_str!("shaders/common/coords.wgsl"),
            "\n",
            include_str!("shaders/common/camera3d.wgsl"),
            "\n",
            include_str!("shaders/common/lighting.wgsl"),
            "\n",
            include_str!("shaders/common/oit.wgsl"),
            "\n",
            include_str!("shaders/common/healpix_scheme.wgsl"),
            "\n",
            include_str!("shaders/common/healpix_math.wgsl"),
            "\n",
            include_str!("shaders/common/healpix_interp.wgsl"),
            "\n",
            include_str!("shaders/common/projections.wgsl"),
            "\n",
            $plot_shader
        )
    };
}

#[cfg(test)]
mod tests {
    /// Builds the 3D renderers (all their pipelines, including the transparent
    /// ones) and fails on any wgpu validation error. Skipped without a GPU.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn mesh_and_point_cloud_pipelines_validate() {
        let (Some((device, _queue)), Some(rt)) = (
            super::test_gpu::device(),
            tokio::runtime::Runtime::new().ok(),
        ) else {
            eprintln!("SKIPPED mesh_and_point_cloud_pipelines_validate: no GPU device");
            return;
        };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let data = [0.0f32; 4];
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let surface = super::SurfaceRenderer::new_surface(&device, format, &data, 2, 2);
        let sphere = super::SphereRenderer::new_sphere(&device, format, &data, 2, 2);
        let points = super::PointCloudRenderer::new(&device, format, &data, 2, 2);
        // OIT pipelines are built on first use; build them here too.
        for mesh in [&surface, &sphere] {
            let _oit = super::oit::OitState::new(&device, format, |variant| {
                mesh.oit_pipeline(&device, variant)
            });
        }
        let _oit = super::oit::OitState::new(&device, format, |variant| {
            points.oit_pipeline(&device, variant)
        });
        if let Some(error) = rt.block_on(scope.pop()) {
            panic!("pipeline validation failed: {error}");
        }
    }

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
                "volume_hardware_filter",
                crate::plots::volume::pipeline::SHADER_HARDWARE_FILTER,
            ),
            (
                "volume_manual_filter",
                crate::plots::volume::pipeline::SHADER_MANUAL_FILTER,
            ),
            (
                "volume_blit",
                concat!(
                    include_str!("shaders/common/fullscreen.wgsl"),
                    "\n",
                    include_str!("shaders/volume/blit.wgsl")
                ),
            ),
            (
                "oit_composite",
                concat!(
                    include_str!("shaders/common/fullscreen.wgsl"),
                    "\n",
                    include_str!("shaders/oit/composite.wgsl")
                ),
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
                    include_str!("shaders/common/healpix_scheme.wgsl"),
                    "\n",
                    include_str!("shaders/common/healpix_math.wgsl"),
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
