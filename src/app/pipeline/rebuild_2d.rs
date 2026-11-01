//! GPU renderers of a layer's 2D matrix data: heatmap, labels, line, and the
//! sphere, surface and 3D coastlines built from the same grid.

use super::rebuild::follow_variable;
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId, MeshGrid};
use crate::app::state::MeshDetail;
use crate::data::MatrixData;
use crate::data::render::decimate::stride_for;
use crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
use crate::plots::limits::max_storage_elements;
use crate::plots::{
    Coastline3DRenderer, CoastlineLod, CoastlineRenderer, HeatmapRenderer, LabelRenderer,
    LineRenderer, PlotType, SphereRenderer, SurfaceRenderer,
};
use eframe::egui_wgpu::RenderState;
use std::sync::Arc;

impl OctantApp {
    /// Rebuilds or updates layer `id`'s GPU buffers for 2D matrix data.
    pub fn rebuild_pipeline_with_matrix_data(&mut self, id: LayerId, data: MatrixData) {
        let is_base = id == LayerId::BASE;
        let step = self.layer_step(id);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let range = (data.min_val, data.max_val);
        let is_new_variable = follow_variable(layer, "2d", Layer::clear_3d, step, range);
        let total_elements = data.width.saturating_mul(data.height);
        if is_base && data.height == 1 {
            let line = &mut self.plot_configs.line;
            line.all_series = false;
            line.profile_dim_idx = 0;
            line.profile_slice_idx = 0;
        }
        // Decided per layer: only this layer's own size forces resampling.
        let resample = needs_pyramid(self.enable_pyramid_resampling, total_elements);
        let pyramid = resample.then_some(self.pyramid_aggregation_op);
        let shown = shown_data(layer, &data, pyramid, &self.nav);
        if let Some(rs) = &self.wgpu_render_state {
            // Coastlines follow the base layer's grid.
            let coastline = is_base.then_some(&mut self.coastline_3d_renderer);
            let base = coastline.map(|slot| (slot, self.coastline_current_lod));
            let reuse = !is_new_variable;
            let mesh_budget = mesh_budget(self.plot_configs.mesh.detail, self.layout.canvas_px);
            let rebuilt = sync_renderers(layer, rs, base, (&shown, &data), mesh_budget, reuse);
            if rebuilt && data.height == 1 {
                // The base layer's plot type is staged in `selected`.
                let plot_type = if is_base {
                    &mut self.selected.plot_type
                } else {
                    &mut layer.selection_mut().plot_type
                };
                follow_row_plot_type(plot_type, data.grid.is_healpix());
            }
        }
        layer.data.matrix = Some(data);
        layer.data.touch_matrix();
        self.ensure_coastline_renderer();
    }

    /// Builds the 2D coastline renderer once, at the coarsest LOD (finer
    /// ones load in the background).
    fn ensure_coastline_renderer(&mut self) {
        if self.coastline_renderer.is_some() {
            return;
        }
        let Some(rs) = &self.wgpu_render_state else {
            return;
        };
        let buf = crate::plots::load_coastline_sync(CoastlineLod::Lod110m);
        let renderer = CoastlineRenderer::new(&rs.device, rs.target_format, buf.as_slice());
        self.coastline_renderer = Some(Arc::new(renderer));
        self.coastline_current_lod = CoastlineLod::Lod110m;
    }
}

/// The data uploaded to the GPU: `data` itself, or with `pyramid` (the
/// aggregation, when resampling) the pyramid's sample of the heatmap view
/// in `nav`, keeping the pyramid on the layer. Composites and single rows are
/// never resampled.
fn shown_data(
    layer: &mut Layer,
    data: &MatrixData,
    pyramid: Option<crate::data::AggregationOp>,
    nav: &crate::app::state::NavigationState,
) -> MatrixData {
    let is_composite = layer.composite.enabled
        || data.dataset_name.contains("RGB Composite")
        || data.dataset_name.contains("CMYK Composite");
    let Some(op) = pyramid.filter(|_| !is_composite && data.height > 1) else {
        layer.data.pyramid = None;
        layer.data.resampler.set_pyramid(None);
        return data.clone();
    };
    let op = layer.pyramid_op(op);
    let pyramid = Arc::new(crate::data::MatrixPyramid::new(
        &data.values,
        data.width,
        data.height,
        &data.dataset_name,
        op,
        512,
    ));
    layer.data.resampler.set_pyramid(Some(pyramid.clone()));
    layer.data.pyramid = Some(pyramid.clone());
    let pan = [nav.heatmap_pan.x, nav.heatmap_pan.y];
    let (u, v) = crate::data::ViewportResampler::compute_visible_data_bounds(
        pan,
        nav.heatmap_zoom,
        [1.0, 1.0],
    );
    let target =
        crate::data::ViewportResampler::compute_target_resolution(data.width, data.height, 2048);
    pyramid.sample_viewport(&data.values, u, v, target)
}

/// Whether a layer of `cells` 2D cells is resampled through a pyramid: when
/// Resampling is ticked (`enabled`), or when it does not fit one GPU storage
/// buffer (`limits::max_storage_elements`, the device's own limit).
pub(crate) fn needs_pyramid(enabled: bool, cells: usize) -> bool {
    enabled || cells > max_storage_elements()
}

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

/// Keeps the layer's label colors, then writes `shown` into its renderers
/// when they can take it in place (`reuse`, same grid and size), else builds
/// them again. The base layer, given with its 3D coastline slot and LOD
/// (`base`), also gets its line renderer, built once: line plots draw only
/// the base layer, from data uploaded at paint time. The sphere and surface
/// draw the whole `data` (not the heatmap's pyramid sample) thinned to at
/// most `mesh_budget` cells (`LayerData::mesh`), as a mesh costs time per
/// cell, whatever its size. Returns whether they were built again.
fn sync_renderers(
    layer: &mut Layer,
    rs: &RenderState,
    base: Option<(&mut Option<Arc<Coastline3DRenderer>>, CoastlineLod)>,
    (shown, data): (&MatrixData, &MatrixData),
    mesh_budget: usize,
    reuse: bool,
) -> bool {
    if base.is_some() && layer.renderers.line.is_none() {
        let line = LineRenderer::new(&rs.device, rs.target_format);
        layer.renderers.line = Some(Arc::new(line));
    }
    let info = layer.source.labels().map(|style| &style.info);
    LabelRenderer::sync(
        &mut layer.renderers.labels,
        info,
        &rs.device,
        rs.target_format,
    );
    // Label images are drawn only as heatmaps.
    // HEALPix grids are never thinned: their meshes take every cell.
    let cells = data.width.saturating_mul(data.height);
    let fits = !data.grid.is_healpix() || cells <= MAX_GPU_STORAGE_BUFFER_ELEMENTS;
    let has_3d = layer.labels().is_none() && fits;
    let stride = stride_for(cells, mesh_budget);
    let thinned = has_3d.then(|| data.decimated(stride)).flatten();
    let mesh = thinned.as_ref().unwrap_or(data);
    let rebuilt = if reuse && can_update(layer, shown, mesh, has_3d) {
        let coastline = base.as_ref().and_then(|(slot, _)| slot.as_deref());
        update_renderers(layer, coastline, rs, shown, mesh);
        false
    } else {
        build_renderers(layer, base, rs, (shown, mesh), has_3d);
        true
    };
    layer.data.mesh = thinned.map(|matrix| MeshGrid { matrix, stride });
    rebuilt
}

/// Whether the layer's renderers can take `shown` (and the sphere and
/// surface its thinned `mesh`) in place: same grid and sizes, with the
/// sphere and surface built when the data has them.
fn can_update(layer: &Layer, shown: &MatrixData, mesh: &MatrixData, has_3d: bool) -> bool {
    let same_dimensions = layer.data.matrix.as_ref().is_some_and(|m| {
        m.grid.same_geometry(&shown.grid) && m.width == shown.width && m.height == shown.height
    });
    let r = &layer.renderers;
    let mesh_fits = |r: &Option<Arc<SurfaceRenderer>>| {
        r.as_ref()
            .is_some_and(|r| (r.width, r.height) == (mesh.width, mesh.height))
    };
    same_dimensions
        && r.heatmap.is_some()
        && (!has_3d || (mesh_fits(&r.sphere) && mesh_fits(&r.surface)))
}

/// Writes `shown`'s coordinates and values into the existing heatmap, and
/// `mesh`'s into the sphere, surface and 3D coastlines.
fn update_renderers(
    layer: &Layer,
    coastline: Option<&Coastline3DRenderer>,
    rs: &RenderState,
    shown: &MatrixData,
    mesh: &MatrixData,
) {
    let q = &rs.queue;
    macro_rules! update {
        ($renderer:expr, $data:expr) => {
            if let Some(r) = $renderer {
                if let Some((cx, cy)) = $data.grid.coords_x().zip($data.grid.coords_y()) {
                    r.update_coords(q, cx, cy);
                }
                r.update_data(q, &$data.values);
            }
        };
    }
    let r = &layer.renderers;
    update!(r.heatmap.as_deref(), shown);
    update!(r.sphere.as_deref(), mesh);
    update!(r.surface.as_deref(), mesh);
    update!(coastline, mesh);
}

/// Builds the heatmap for `shown`, and with `has_3d` the sphere, surface
/// and (for the base layer, given its 3D coastline slot and LOD, `base`) 3D
/// coastlines for `mesh`.
fn build_renderers(
    layer: &mut Layer,
    base: Option<(&mut Option<Arc<Coastline3DRenderer>>, CoastlineLod)>,
    rs: &RenderState,
    (shown, mesh): (&MatrixData, &MatrixData),
    has_3d: bool,
) {
    let (device, format) = (&rs.device, rs.target_format);
    let (values, w, h) = (&shown.values, shown.width, shown.height);
    let (cx, cy) = (shown.grid.coords_x(), shown.grid.coords_y());
    let renderers = &mut layer.renderers;
    let heatmap = HeatmapRenderer::new_with_coords(device, format, values, w, h, cx, cy);
    renderers.heatmap = Some(Arc::new(heatmap));
    if !has_3d {
        renderers.sphere = None;
        renderers.surface = None;
        if let Some((slot, _)) = base {
            *slot = None;
        }
        return;
    }
    let (values, w, h) = (&mesh.values, mesh.width, mesh.height);
    let (cx, cy) = (mesh.grid.coords_x(), mesh.grid.coords_y());
    let sphere = SphereRenderer::new_sphere_with_coords(device, format, values, w, h, cx, cy);
    let surface = SurfaceRenderer::new_surface_with_coords(device, format, values, w, h, cx, cy);
    renderers.sphere = Some(Arc::new(sphere));
    renderers.surface = Some(Arc::new(surface));
    if let Some((slot, lod)) = base {
        let buf = crate::plots::load_coastline_sync(lod);
        let coast = Coastline3DRenderer::new(device, format, buf.as_slice(), values, w, h, cx, cy);
        *slot = Some(Arc::new(coast));
    }
}

/// A single row plots as a line, except on HEALPix grids, which keep a
/// heatmap (switched to one from a line).
fn follow_row_plot_type(plot_type: &mut PlotType, is_healpix: bool) {
    if !is_healpix {
        *plot_type = PlotType::Line;
    } else if *plot_type == PlotType::Line {
        *plot_type = PlotType::Heatmap;
    }
}
