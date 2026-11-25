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
pub(crate) fn mesh_budget(detail: MeshDetail, [w, h]: [u32; 2]) -> usize {
    let pixels = (w as usize).saturating_mul(h as usize);
    detail
        .budget(pixels)
        .clamp(MIN_MESH_CELLS, MAX_GPU_STORAGE_BUFFER_ELEMENTS)
}

/// `data` thinned to at most `budget` cells for the sphere and surface;
/// `None` when they draw every cell.
pub(crate) fn mesh_grid(data: &MatrixData, budget: usize) -> Option<MeshGrid> {
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
    rethin_detail_sphere(layer, rs, budget);
    let Some(matrix) = layer.data.matrix.as_ref() else {
        return;
    };
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

/// Thins `layer`'s detail sphere to `budget` cells when one is resident
/// and its stride changes.
fn rethin_detail_sphere(layer: &mut Layer, rs: &RenderState, budget: usize) {
    let (Some(tile), Some(_)) = (&mut layer.data.detail.tile, &layer.renderers.detail_sphere)
    else {
        return;
    };
    let Some(matrix) = tile.matrix.as_deref() else {
        return;
    };
    let stride = stride_for(matrix.width, matrix.height, budget);
    if tile.thinning.stride.max(1) as usize == stride {
        return;
    }
    let grid = mesh_grid(matrix, budget);
    tile.thinning = MeshGrid::thinning(grid.as_ref(), matrix);
    let m = grid.as_ref().map_or(matrix, |g| &g.matrix);
    sync_detail_sphere(&mut layer.renderers.detail_sphere, rs, m);
    layer.data.touch_matrix();
}

/// Uploads `matrix` to `layer`'s active detail renderer (the sphere's when
/// `sphere` is true, else the heatmap's); headless (`rs` is `None`), only
/// computes the sphere's [`crate::plots::Thinning`].
pub(crate) fn upload_detail_renderers(
    layer: &mut Layer,
    rs: Option<&RenderState>,
    sphere: bool,
    budget: usize,
    matrix: &MatrixData,
    uv: [f32; 4],
) -> crate::plots::Thinning {
    let mesh = sphere.then(|| mesh_grid(matrix, budget)).flatten();
    let thinning = MeshGrid::thinning(mesh.as_ref(), matrix);
    let Some(rs) = rs else {
        return thinning;
    };
    if sphere && layer.labels().is_none() {
        let m = mesh.as_ref().map_or(matrix, |g| &g.matrix);
        sync_detail_sphere(&mut layer.renderers.detail_sphere, rs, m);
    } else {
        let (w, h) = (matrix.width, matrix.height);
        let r = match &layer.renderers.detail {
            Some(r) if r.capacity() >= w.saturating_mul(h) => r.clone(),
            _ => Arc::new(crate::plots::HeatmapRenderer::new(
                &rs.device,
                rs.target_format,
                &matrix.values,
                w,
                h,
            )),
        };
        r.update_data_and_dimensions(&rs.queue, &matrix.values, w, h, uv);
        layer.renderers.detail = Some(r);
    }
    thinning
}

fn sync_detail_sphere(
    slot: &mut Option<Arc<crate::plots::SphereRenderer>>,
    rs: &RenderState,
    m: &MatrixData,
) {
    let (sw, sh) = (m.width, m.height);
    let r = match slot {
        Some(r) if (r.width, r.height) == (sw, sh) => {
            r.update_data(&rs.queue, &m.values);
            r.clone()
        }
        _ => Arc::new(crate::plots::SphereRenderer::new_sphere_with_coords(
            &rs.device,
            rs.target_format,
            &m.values,
            sw,
            sh,
            None,
            None,
        )),
    };
    *slot = Some(r);
}
