//! A detail tile's request and its upload: the level's selection over the
//! tile's cells, then its block sliced like a plotted one and drawn by its
//! layer's detail renderer at the tile's place on the plotted quad.

use std::sync::Arc;

use super::{DetailTile, MARGIN, PendingDetail, TilePlan, Want};
use crate::app::OctantApp;
use crate::app::block_loading::block_axes;
use crate::app::block_loading::projection_2d::PlaneWindow;
use crate::app::layers::LayerId;
use crate::app::multiscale::to_level;
use crate::data::slicing::CompositeProbe;
use crate::data::{BlockCacheKey, MatrixData, OctantBlock};
use crate::plots::HeatmapRenderer;

impl OctantApp {
    /// `want`'s tile: the request reading its cells (the current step of an
    /// animated dimension) and the store it reads from.
    pub(super) fn plan_detail(&self, want: &Want) -> Option<TilePlan> {
        let mut sel = to_level(self.plotted(), want.level)?;
        let var = sel.variable_info()?.clone();
        let mut cover = [(0.0, 0.0); 2];
        let mut origin = [0; 2];
        let mut end = [0; 2];
        for (i, axis) in want.axes.iter().enumerate() {
            let chunk = var.chunk_shape.get(axis.dim).copied().unwrap_or(1);
            let cells = axis.cells(&want.set, want.level, want.visible[i], MARGIN, chunk)?;
            cover[i] = axis.cover(&want.set, want.level, &cells)?;
            origin[i] = cells.start;
            end[i] = cells.end;
            *sel.dim_ranges.get_mut(axis.dim)? = (cells.start as usize, cells.end as usize - 1);
        }
        let (u0, u1) = want.axes[0].quad_fraction(cover[0]);
        let (v0, v1) = want.axes[1].quad_fraction(cover[1]);
        let tile = DetailTile {
            level: want.level,
            var_idx: sel.variable_idx,
            dims: [want.axes[0].dim, want.axes[1].dim],
            origin,
            end,
            matrix: None,
            probe: None,
            context: want.context,
            cover,
            uv: [u0, v0, u1, v1],
        };
        self.plan_tile(LayerId::BASE, sel, tile)
    }

    /// The request reading `tile` for layer `id` from `sel` (the tile's
    /// level, its x and y ranges set), at the layer's current step of an
    /// animated dimension.
    pub(super) fn plan_tile(
        &self,
        id: LayerId,
        mut sel: crate::app::layers::VariableSelection,
        tile: DetailTile,
    ) -> Option<TilePlan> {
        let var = sel.variable_info()?.clone();
        let step = self.layer_step(id);
        if let Some(a) = sel.animated_dim {
            let last = (var.shape.get(a).copied().unwrap_or(1) as usize).saturating_sub(1);
            let step = step.min(last);
            *sel.dim_ranges.get_mut(a)? = (step, step);
            *sel.dim_indices.get_mut(a)? = step;
        }
        let slice = sel.slice_request(&var.name, &var.shape);
        let source_id = self.selection_source_id(&sel);
        let handle = self.resolve_store_handle(&source_id, &sel.store_target, sel.store_kind)?;
        let pending = PendingDetail {
            layer: id,
            key: BlockCacheKey::new(source_id, &slice),
            tile,
            dims: var.dimension_names,
            config: sel.dim_config,
            indices: sel.dim_indices,
            ranges: sel.dim_ranges,
            anim_dim: sel.animated_dim,
            step,
        };
        Some((pending, slice, handle))
    }

    /// Slices `block` into `pending`'s tile and draws it; nothing when the
    /// block runs in another orientation than the plot.
    pub(super) fn show_detail(&mut self, pending: PendingDetail, block: &OctantBlock) {
        let Some((matrix, probe)) = self.slice_tile(&pending, block) else {
            log::debug!("Detail tile of '{}' skipped", block.variable_name);
            return;
        };
        if pending.layer == LayerId::BASE {
            self.request_detail_coordinates(pending.tile.var_idx);
        }
        let rs = self.wgpu_render_state.clone();
        let Some(layer) = self.layers.get_mut(pending.layer) else {
            return;
        };
        let mut tile = pending.tile;
        // Headless, the tile is decided and nothing draws it.
        if let Some(rs) = rs {
            let (w, h) = (matrix.width, matrix.height);
            let renderer = match &layer.renderers.detail {
                Some(r) if r.capacity() >= w.saturating_mul(h) => r.clone(),
                _ => Arc::new(HeatmapRenderer::new(
                    &rs.device,
                    rs.target_format,
                    &matrix.values,
                    w,
                    h,
                )),
            };
            renderer.update_data_and_dimensions(&rs.queue, &matrix.values, w, h, tile.uv);
            layer.renderers.detail = Some(renderer);
        }
        tile.matrix = Some(Arc::new(matrix));
        tile.probe = probe;
        layer.data.detail.tile = Some(tile);
    }

    /// `block` sliced into `pending`'s tile like a plotted block (a composite
    /// with its plot's stretch, and the window behind it); `None` when it runs
    /// in another orientation than its layer's plot.
    fn slice_tile(
        &self,
        pending: &PendingDetail,
        block: &OctantBlock,
    ) -> Option<(MatrixData, Option<CompositeProbe>)> {
        let layer = self.layers.get(pending.layer)?;
        let flips = &layer.data.flipped_dims;
        let same_flips = flips.len() == block.flipped_dims.len()
            && block.flipped_dims.iter().all(|f| flips.contains(f));
        if !same_flips || block.dimension_names != pending.dims {
            return None;
        }
        let (dims, ranges) = (&pending.dims, &pending.ranges);
        let (bx, by, _) =
            Self::resolve_spatial_axes(block.rank(), &block.dimension_names, dims, &pending.config);
        let fixed = block_axes::fixed_indices(
            block,
            dims,
            pending.anim_dim,
            &pending.indices,
            pending.step,
        );
        let (_, local_x) = block_axes::dim_bounds(block, dims, bx, ranges);
        let (_, local_y) = block_axes::dim_bounds(block, dims, by, ranges);
        let window = PlaneWindow {
            x_dim: bx,
            y_dim: by,
            x_range: block.oriented_range(bx, local_x),
            y_range: block.oriented_range(by, local_y),
            fixed: &fixed,
        };
        if layer.composite.enabled {
            let stretch = layer.data.composite_stretch.clone().unwrap_or_default();
            let c_dim = self.layer_channel_dim(pending.layer).unwrap_or(0);
            if let Some(c) = self.slice_composite(pending.layer, block, c_dim, &window, stretch) {
                return Some((c.matrix, Some(c.probe)));
            }
        }
        let matrix = block.slice_2d_with_ranges(
            window.x_dim,
            window.y_dim,
            window.x_range,
            window.y_range,
            window.fixed,
            self.layer_animated_dim_extent(pending.layer),
            &block.variable_name,
            false,
        )?;
        Some((matrix, None))
    }

    /// Reads the coordinates of the tile's level variable `idx` in the
    /// background (once per variable), for the hover.
    fn request_detail_coordinates(&mut self, idx: usize) {
        if let Some((source_id, store, var)) = self.coordinate_request(self.plotted(), idx) {
            self.coordinate_loader.request(&source_id, store, var);
        }
    }
}
