pub mod coastline;
pub mod colormap_atlas;
pub mod common;
pub mod device;
pub mod fullscreen;
pub mod heatmap;
mod heatmap_draw;
pub mod labels;
#[cfg(test)]
mod labels_tests;
pub mod line;
pub mod mesh;
mod mesh_draw;
pub mod oit;
pub mod point_cloud;
mod point_cloud_draw;
mod point_cloud_types;
mod quad;
#[cfg(test)]
mod shader_tests;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) mod test_gpu;
pub mod traits;
pub mod volume;

use crate::i18n::{PlotKey, tr};

pub use coastline::{
    Coastline3DCallback, Coastline3DParams, Coastline3DRenderer, CoastlineBuffer,
    CoastlineCallback, CoastlineFetchResult, CoastlineLod, CoastlineReceiver, CoastlineRenderer,
    CoastlineSender, dataset_geo_bounds, expand_coastline_line_list, fetch_coastline_async,
    load_coastline_sync,
};
pub use common::{Mesh3DUniformParams, Mesh3DUniforms, MeshVertex3D, PlotColorParams};
pub use heatmap::HeatmapRenderer;
pub use heatmap_draw::HeatmapCallback;
pub use labels::{LabelDraw, LabelRenderer};
pub use line::{LineCallback, LineRenderer, LineShape};
pub use mesh::Mesh3DRenderer;
pub use mesh_draw::{
    Mesh3DCallback, SphereCallback, SphereRenderer, SurfaceCallback, SurfaceRenderer,
};
pub use point_cloud::{PointCloudCallback, PointCloudRenderer, PointCloudUniformParams};
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
            PlotType::Heatmap => tr(PlotKey::Heatmap),
            PlotType::Line => tr(PlotKey::Line),
            PlotType::Surface => tr(PlotKey::Surface),
            PlotType::Volume => tr(PlotKey::Volume),
            PlotType::Sphere => tr(PlotKey::Sphere),
            PlotType::PointCloud => tr(PlotKey::PointCloud),
        }
    }

    /// Whether the coastline overlay is drawn over this plot type.
    pub fn draws_coastlines(&self) -> bool {
        matches!(
            self,
            PlotType::Heatmap | PlotType::Surface | PlotType::Sphere
        )
    }

    /// Whether categorical colors apply. Volumes and point clouds draw no
    /// categories: their data has no category set, and DVR blends values
    /// across voxels.
    pub fn draws_categories(&self) -> bool {
        !matches!(self, PlotType::Volume | PlotType::PointCloud)
    }

    /// Whether the plot is drawn in 3D, with an orbiting camera.
    pub fn is_3d(&self) -> bool {
        matches!(
            self,
            PlotType::Sphere | PlotType::Surface | PlotType::Volume | PlotType::PointCloud
        )
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
            include_str!("shaders/common/height.wgsl"),
            "\n",
            $plot_shader
        )
    };
}
