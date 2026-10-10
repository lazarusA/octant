//! Plot controllers managing plot-specific camera interaction, zoom/pan navigation, and capabilities.

mod heatmap;
mod line;
mod mesh;
mod point_cloud;
#[cfg(test)]
mod tests;
mod traits;
mod volume;

pub use heatmap::HeatmapController;
pub use line::LineController;
pub use mesh::MeshController;
pub use point_cloud::PointCloudController;
pub use traits::PlotController;
pub use volume::VolumeController;

use crate::plots::PlotType;

static HEATMAP_CONTROLLER: HeatmapController = HeatmapController;
static LINE_CONTROLLER: LineController = LineController;
static VOLUME_CONTROLLER: VolumeController = VolumeController;
static SURFACE_CONTROLLER: MeshController = MeshController::new(PlotType::Surface);
static SPHERE_CONTROLLER: MeshController = MeshController::new(PlotType::Sphere);
static POINT_CLOUD_CONTROLLER: PointCloudController = PointCloudController;

/// Returns the singleton controller for the specified plot type.
pub fn controller_for(plot_type: PlotType) -> &'static dyn PlotController {
    match plot_type {
        PlotType::Heatmap => &HEATMAP_CONTROLLER,
        PlotType::Line => &LINE_CONTROLLER,
        PlotType::Volume => &VOLUME_CONTROLLER,
        PlotType::Surface => &SURFACE_CONTROLLER,
        PlotType::Sphere => &SPHERE_CONTROLLER,
        PlotType::PointCloud => &POINT_CLOUD_CONTROLLER,
    }
}
