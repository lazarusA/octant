//! How many cells the sphere and surface draw (`MeshDetail`): grids with
//! more are thinned (`MatrixData::decimated`).

use super::rebuild_2d::build_meshes;
use crate::app::MeshDetail;
use crate::app::OctantApp;
use crate::app::layers::MeshGrid;
use crate::data::render::decimate::stride_for;
use crate::plots::PlotType;
use crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;

/// Cells a mesh may always draw, however small the canvas.
const MIN_MESH_CELLS: usize = 256 * 1024;

/// The most cells the sphere and surface draw at `detail` on a canvas of
/// `canvas_px`, within one storage buffer.
pub(super) fn mesh_budget(detail: MeshDetail, [w, h]: [u32; 2]) -> usize {
    let pixels = (w as usize).saturating_mul(h as usize);
    detail
        .budget(pixels)
        .clamp(MIN_MESH_CELLS, MAX_GPU_STORAGE_BUFFER_ELEMENTS)
}

impl OctantApp {
    /// Records the canvas's size in pixels `px`; meshes and point clouds
    /// draw as many cells as it shows (`mesh_budget`), so a new size thins
    /// the shown one again.
    pub fn record_canvas_size(&mut self, px: egui::Vec2) {
        let canvas_px = [px.x.round() as u32, px.y.round() as u32];
        if px.x < 1.0 || px.y < 1.0 || canvas_px == self.layout.canvas_px {
            return;
        }
        self.layout.canvas_px = canvas_px;
        match self.effective_canvas_plot_type() {
            PlotType::Surface | PlotType::Sphere => self.rethin_meshes(),
            // The point cloud is built with the volume, so a volume shown
            // now has it ready at this size when switched to.
            PlotType::PointCloud | PlotType::Volume => self.rethin_points(),
            _ => {}
        }
    }

    /// Thins the base layer's sphere and surface again after the Detail
    /// setting or the canvas size changed, when that changes their stride.
    pub fn rethin_meshes(&mut self) {
        let budget = mesh_budget(self.plot_configs.mesh.detail, self.layout.canvas_px);
        let base = &mut self.layers.base;
        let Some(matrix) = base.data.matrix.as_ref() else {
            return;
        };
        if matrix.height < 2 || matrix.grid.is_healpix() || base.labels().is_some() {
            return;
        }
        let stride = stride_for(matrix.width, matrix.height, budget);
        // An unchanged mesh is kept: writing it again would overwrite the
        // colors a GPU composite baked into it.
        if base.data.mesh.as_ref().map_or(1, |m| m.stride) == stride {
            return;
        }
        let Some(rs) = &self.wgpu_render_state else {
            return;
        };
        // Only the meshes change: the heatmap and its pyramid stay.
        let Some(matrix) = base.data.matrix.take() else {
            return;
        };
        let thinned = matrix.decimated(stride);
        let coastline = Some((&mut self.coastline_3d_renderer, self.coastline_current_lod));
        build_meshes(base, coastline, rs, thinned.as_ref().unwrap_or(&matrix));
        base.data.matrix = Some(matrix);
        base.data.mesh = thinned.map(|matrix| MeshGrid { matrix, stride });
        base.data.touch_matrix();
    }
}
