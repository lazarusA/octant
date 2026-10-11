//! Plot controllers managing plot-specific camera interaction, zoom/pan navigation, and capabilities.

mod orbit;
mod planar;
#[cfg(test)]
mod tests;
mod traits;

pub use orbit::Orbit3DController;
pub use planar::{HeatmapController, LineController};
pub use traits::PlotController;

use crate::plots::PlotType;

static HEATMAP_CONTROLLER: HeatmapController = HeatmapController;
static LINE_CONTROLLER: LineController = LineController;
static VOLUME_CONTROLLER: Orbit3DController = Orbit3DController::new(PlotType::Volume);
static SURFACE_CONTROLLER: Orbit3DController = Orbit3DController::new(PlotType::Surface);
static SPHERE_CONTROLLER: Orbit3DController = Orbit3DController::new(PlotType::Sphere);
static POINT_CLOUD_CONTROLLER: Orbit3DController = Orbit3DController::new(PlotType::PointCloud);

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
