//! The GPU renderers a layer draws with, one per plot type, built on first use.

use std::sync::Arc;

use crate::plots::{
    LineRenderer, MatrixRenderer, PlotType, PointCloudRenderer, SphereRenderer, SurfaceRenderer,
    VolumeRenderer,
};

#[derive(Default)]
pub struct LayerRenderers {
    pub heatmap: Option<Arc<MatrixRenderer>>,
    pub line: Option<Arc<LineRenderer>>,
    pub sphere: Option<Arc<SphereRenderer>>,
    pub surface: Option<Arc<SurfaceRenderer>>,
    pub volume: Option<Arc<VolumeRenderer>>,
    pub point_cloud: Option<Arc<PointCloudRenderer>>,
    /// Z planes of the layer's volume not yet uploaded to the volume renderer.
    pub volume_dirty: Option<std::ops::Range<usize>>,
    /// Z planes of the layer's volume not yet uploaded to the point cloud renderer.
    pub point_cloud_dirty: Option<std::ops::Range<usize>>,
}

impl LayerRenderers {
    /// Frees the OIT frames of 3D renderers not on screen (`active` is the
    /// plot type drawn this frame).
    pub fn release_idle_oit_frames(&self, active: PlotType) {
        let meshes = [
            (PlotType::Sphere, &self.sphere),
            (PlotType::Surface, &self.surface),
        ];
        for (kind, renderer) in meshes {
            if kind != active
                && let Some(renderer) = renderer
            {
                renderer.oit.release();
            }
        }
        if active != PlotType::PointCloud
            && let Some(renderer) = &self.point_cloud
        {
            renderer.oit.release();
        }
    }
}
