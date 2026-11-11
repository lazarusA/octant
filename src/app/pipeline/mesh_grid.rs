//! How many cells the sphere and surface draw (`MeshDetail`): grids with
//! more are thinned (`MatrixData::decimated`).

use super::rebuild_2d::build_meshes;
use crate::app::MeshDetail;
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId, MeshGrid};
use crate::data::MatrixData;
use crate::data::render::decimate::stride_for;
use crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
use crate::plots::{Coastline3DRenderer, CoastlineLod, PlotType};
use eframe::egui_wgpu::RenderState;
use std::sync::Arc;

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

/// `data` thinned to at most `budget` cells for the sphere and surface;
/// `None` when they draw every cell.
pub(super) fn mesh_grid(data: &MatrixData, budget: usize) -> Option<MeshGrid> {
    let stride = stride_for(data.width, data.height, budget);
    data.decimated(stride)
        .map(|matrix| MeshGrid { matrix, stride })
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

    /// Thins every layer's sphere and surface again after the Detail
    /// setting or the canvas size changed, when that changes their stride:
    /// overlays keep the base's grid, which their heights are read from.
    pub fn rethin_meshes(&mut self) {
        let budget = mesh_budget(self.plot_configs.mesh.detail, self.layout.canvas_px);
        let Some(rs) = &self.wgpu_render_state else {
            return;
        };
        let lod = self.coastline_current_lod;
        let mut coastline = Some(&mut self.coastline_3d_renderer);
        for layer in self.layers.iter_mut() {
            // The 3D coastlines follow the base layer, drawn first.
            let slot = coastline.take().filter(|_| layer.id() == LayerId::BASE);
            rethin_layer(layer, slot.map(|slot| (slot, lod)), rs, budget);
        }
    }
}

/// Thins `layer`'s sphere and surface to `budget` cells, when that changes
/// their stride (and its 3D coastlines, given their slot and LOD).
fn rethin_layer(
    layer: &mut Layer,
    coastline: Option<(&mut Option<Arc<Coastline3DRenderer>>, CoastlineLod)>,
    rs: &RenderState,
    budget: usize,
) {
    let Some(matrix) = layer.data.matrix.as_ref() else {
        return;
    };
    if matrix.height < 2 || matrix.grid.is_healpix() || layer.labels().is_some() {
        return;
    }
    // An unchanged mesh is kept: writing it again would overwrite the
    // colors a GPU composite baked into it.
    let stride = stride_for(matrix.width, matrix.height, budget);
    if layer.data.mesh.as_ref().map_or(1, |m| m.stride) == stride {
        return;
    }
    // Only the meshes change: the heatmap and its pyramid stay.
    let Some(matrix) = layer.data.matrix.take() else {
        return;
    };
    let grid = mesh_grid(&matrix, budget);
    build_meshes(
        layer,
        coastline,
        rs,
        grid.as_ref().map_or(&matrix, |g| &g.matrix),
    );
    layer.data.matrix = Some(matrix);
    layer.data.mesh = grid;
    layer.data.touch_matrix();
}
