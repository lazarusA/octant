//! How many cells the sphere and surface draw (`MeshDetail`): grids with
//! more are thinned (`MatrixData::decimated`).

use crate::app::MeshDetail;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::render::decimate::stride_for;
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
    /// Thins the base layer's sphere and surface again after the Detail
    /// setting changed, when that changes their stride.
    pub fn rethin_meshes(&mut self) {
        let budget = mesh_budget(self.plot_configs.mesh.detail, self.layout.canvas_px);
        let base = &mut self.layers.base;
        let Some(matrix) = base.data.matrix.as_ref() else {
            return;
        };
        if matrix.height < 2 || matrix.grid.is_healpix() {
            return;
        }
        let stride = stride_for(matrix.width.saturating_mul(matrix.height), budget);
        // An unchanged mesh is kept: writing it again would overwrite the
        // colors a GPU composite baked into it.
        if base.data.mesh.as_ref().map_or(1, |m| m.stride) == stride {
            return;
        }
        if let Some(matrix) = base.data.matrix.take() {
            self.rebuild_pipeline_with_matrix_data(LayerId::BASE, matrix);
        }
    }
}
