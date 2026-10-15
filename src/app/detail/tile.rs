//! A detail tile's request and its upload: the level's selection over the
//! tile's cells, then its block sliced like a plotted one and drawn by the
//! base layer's detail renderer at the tile's place on the plotted quad.

use std::sync::Arc;

use super::{DetailTile, MARGIN, PendingDetail, Want};
use crate::app::OctantApp;
use crate::app::block_loading::block_axes;
use crate::app::layers::LayerId;
use crate::app::multiscale::{at_base, at_level};
use crate::data::{BlockCacheKey, OctantBlock, SliceRequest, StoreHandle};
use crate::plots::HeatmapRenderer;

impl OctantApp {
    /// `want`'s tile: the request reading its cells (the current step of an
    /// animated dimension) and the store it reads from.
    pub(super) fn plan_detail(
        &self,
        want: &Want,
    ) -> Option<(PendingDetail, SliceRequest, StoreHandle)> {
        let plotted = self.plotted();
        let base = match want.from {
            0 => plotted.clone(),
            _ => at_base(plotted)?,
        };
        let mut sel = match want.level {
            0 => base,
            level => at_level(&base, level)?,
        };
        let var = sel.variable_info()?.clone();
        let mut cover = [(0.0, 0.0); 2];
        for (i, axis) in want.axes.iter().enumerate() {
            let chunk = var.chunk_shape.get(axis.dim).copied().unwrap_or(1);
            let cells = axis.cells(&want.set, want.level, want.visible[i], MARGIN, chunk)?;
            cover[i] = axis.cover(&want.set, want.level, &cells)?;
            *sel.dim_ranges.get_mut(axis.dim)? = (cells.start as usize, cells.end as usize - 1);
        }
        let step = self.playback.current_timestep;
        if let Some(a) = sel.animated_dim {
            let last = (var.shape.get(a).copied().unwrap_or(1) as usize).saturating_sub(1);
            let step = step.min(last);
            *sel.dim_ranges.get_mut(a)? = (step, step);
            *sel.dim_indices.get_mut(a)? = step;
        }
        let slice = sel.slice_request(&var.name, &var.shape, None);
        let source_id = self.selection_source_id(&sel);
        let handle = self.resolve_store_handle(&source_id, &sel.store_target, sel.store_kind)?;
        let (u0, u1) = want.axes[0].quad_fraction(cover[0]);
        let (v0, v1) = want.axes[1].quad_fraction(cover[1]);
        let pending = PendingDetail {
            key: BlockCacheKey::new(source_id, &slice),
            tile: DetailTile {
                level: want.level,
                context: want.context,
                cover,
                uv: [u0, v0, u1, v1],
            },
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
        let same_flips = {
            let flips = &self.layers.base.data.flipped_dims;
            flips.len() == block.flipped_dims.len()
                && block.flipped_dims.iter().all(|f| flips.contains(f))
        };
        if !same_flips || block.dimension_names != pending.dims {
            log::debug!(
                "Detail tile of '{}' skipped: block orientation differs",
                block.variable_name
            );
            return;
        }
        let (bx, by, _) = Self::resolve_spatial_axes(
            block.rank(),
            &block.dimension_names,
            &pending.dims,
            &pending.config,
        );
        let fixed = block_axes::fixed_indices(
            block,
            &pending.dims,
            pending.anim_dim,
            &pending.indices,
            pending.step,
        );
        let (_, local_x) = block_axes::dim_bounds(block, &pending.dims, bx, &pending.ranges);
        let (_, local_y) = block_axes::dim_bounds(block, &pending.dims, by, &pending.ranges);
        let extent = self.layer_animated_dim_extent(LayerId::BASE);
        let Some(matrix) = block.slice_2d_with_ranges(
            bx,
            by,
            block.oriented_range(bx, local_x),
            block.oriented_range(by, local_y),
            &fixed,
            extent,
            &block.variable_name,
            false,
        ) else {
            return;
        };
        let Some(rs) = &self.wgpu_render_state else {
            // Headless: the tile is decided, nothing draws it.
            self.layers.base.data.detail.tile = Some(pending.tile);
            return;
        };
        let (w, h) = (matrix.width, matrix.height);
        let cells = w.saturating_mul(h);
        let renderers = &mut self.layers.base.renderers;
        let renderer = match &renderers.detail {
            Some((renderer, capacity)) if *capacity >= cells => renderer.clone(),
            _ => {
                let renderer =
                    HeatmapRenderer::new(&rs.device, rs.target_format, &matrix.values, w, h);
                let renderer = Arc::new(renderer);
                let capacity = cells
                    .max(2048 * 2048)
                    .min(crate::plots::limits::max_storage_elements().max(cells));
                renderers.detail = Some((renderer.clone(), capacity));
                renderer
            }
        };
        renderer.update_data_and_dimensions(&rs.queue, &matrix.values, w, h, pending.tile.uv);
        self.layers.base.data.detail.tile = Some(pending.tile);
    }
}
