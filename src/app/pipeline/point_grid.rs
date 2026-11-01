//! How many points the point cloud draws: as the sphere and surface
//! (`MeshDetail`, `mesh_budget`), since a point costs time on the GPU
//! whether or not the screen has a pixel for it. Volumes with more voxels
//! are thinned along x and y (`VolumeData::thinned_planes`); Z stays whole,
//! so ring steps and plane uploads map one to one.

use std::sync::Arc;

use super::mesh_grid::mesh_budget;
use crate::app::layers::PointGrid;
use crate::app::{OctantApp, SpatialRole};
use crate::data::VolumeData;
use crate::data::render::decimate::stride_for;
use crate::plots::{PointCloudRenderer, Thinning};

impl OctantApp {
    /// The base layer's point grid for a volume of `[w, h, d]` voxels,
    /// `None` when every voxel fits the budget, or when x or y is animated
    /// (their ring shifts move whole voxels).
    pub(crate) fn point_grid(&self, [w, h, d]: [usize; 3]) -> Option<PointGrid> {
        let role = self
            .effective_animated_dim()
            .and_then(|anim| self.effective_dim_config().get(anim))
            .map(|c| c.spatial);
        if matches!(
            role,
            Some(SpatialRole::X | SpatialRole::Y | SpatialRole::Grid)
        ) {
            return None;
        }
        let budget = mesh_budget(self.plot_configs.mesh.detail, self.layout.canvas_px);
        let stride = stride_for(w, h, (budget / d.max(1)).max(1));
        (stride > 1).then(|| PointGrid {
            stride,
            width: w.div_ceil(stride),
            height: h.div_ceil(stride),
        })
    }

    /// Builds the base layer's point cloud again after the Detail setting
    /// or the canvas size changed, when that changes its grid.
    pub fn rethin_points(&mut self) {
        let base = &self.layers.base;
        let Some(v) = &base.data.volume else {
            return;
        };
        let grid = self.point_grid([v.width, v.height, v.depth]);
        if grid == base.data.points || base.renderers.point_cloud.is_none() {
            return;
        }
        let Some(rs) = &self.wgpu_render_state else {
            return;
        };
        let base = &mut self.layers.base;
        let Some(volume) = &base.data.volume else {
            return;
        };
        let renderer = point_cloud_renderer(rs, volume, grid);
        base.renderers.point_cloud = Some(Arc::new(renderer));
        base.renderers.point_cloud_dirty = None;
        base.data.points = grid;
        base.data.touch_volume();
    }
}

/// A point cloud renderer drawing `volume`, thinned by `grid`.
pub(super) fn point_cloud_renderer(
    rs: &eframe::egui_wgpu::RenderState,
    volume: &VolumeData,
    grid: Option<PointGrid>,
) -> PointCloudRenderer {
    let (device, format) = (&rs.device, rs.target_format);
    let size = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let Some(g) = grid else {
        let (w, h) = (size(volume.width), size(volume.height));
        return PointCloudRenderer::new(device, format, &volume.values, w, h);
    };
    let values = volume.thinned_planes(g.stride, 0..volume.depth);
    let renderer = PointCloudRenderer::new(device, format, &values, size(g.width), size(g.height));
    renderer.with_thinning(Thinning {
        stride: size(g.stride),
        width: size(g.width),
        height: size(g.height),
        full_width: size(volume.width),
        full_height: size(volume.height),
    })
}
