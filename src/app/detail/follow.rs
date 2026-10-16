//! Overlays' detail tiles follow the base layer's: as soon as the base
//! layer asks for a tile, each drawn multiscale overlay reads the level of
//! its own stack on that tile's grid over the same cells, drawn at the same
//! place on the quad.

use std::hash::{Hash, Hasher};

use super::{DetailTile, PendingDetail};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::multiscale::{at_base, at_level, level_matching};
use crate::data::{SliceRequest, StoreHandle};

impl OctantApp {
    /// Keeps every overlay's tile on the base layer's latest one (pending,
    /// else shown): asks again when that tile or the overlay's selection
    /// changes, drops tiles of a selection the overlay no longer shows and
    /// every overlay tile once the base layer has none.
    pub(super) fn follow_detail_overlays(&mut self) {
        let base = &self.layers.base.data.detail;
        let target = base
            .pending
            .as_ref()
            .map(|p| &p.tile)
            .or(base.tile.as_ref());
        let target = target.map(|t| (t.clone(), tile_key(t)));
        let ids: Vec<LayerId> = self.layers.overlays().iter().map(|l| l.id()).collect();
        for id in ids {
            let lost = self.overlay_pending_lost(id);
            let context = target
                .as_ref()
                .and_then(|(_, k)| self.overlay_context(id, *k));
            let Some(state) = self.layers.get_mut(id).map(|l| &mut l.data.detail) else {
                continue;
            };
            if lost {
                state.pending = None;
                state.follows = None;
            }
            let Some(context) = context else {
                state.clear();
                continue;
            };
            if state.tile.as_ref().is_some_and(|t| t.context != context) {
                state.tile = None;
            }
            if state.follows == Some(context) {
                continue;
            }
            state.follows = Some(context);
            state.pending = None;
            let plan = target
                .as_ref()
                .and_then(|(tile, _)| self.plan_overlay_tile(id, tile, context));
            if let Some(plan) = plan {
                self.request_tile(plan);
            }
        }
    }

    /// What overlay `id`'s tile depends on: the base tile (`base_key`) and
    /// the overlay's selection and step; `None` when it is not drawn.
    fn overlay_context(&self, id: LayerId, base_key: u64) -> Option<u64> {
        let layer = self.layers.get(id).filter(|l| l.is_drawn())?;
        let sel = layer.selection();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        base_key.hash(&mut h);
        sel.metadata_generation.hash(&mut h);
        sel.variable_idx.hash(&mut h);
        sel.dim_ranges.hash(&mut h);
        sel.dim_indices.hash(&mut h);
        self.layer_step(id).hash(&mut h);
        layer.data.flipped_dims.hash(&mut h);
        Some(h.finish())
    }

    /// Whether overlay `id`'s fetch was lost (aborted, neither pending nor
    /// cached).
    fn overlay_pending_lost(&self, id: LayerId) -> bool {
        let pending = self
            .layers
            .get(id)
            .and_then(|l| l.data.detail.pending.as_ref());
        pending.is_some_and(|p| {
            !self.block_prefetcher.is_pending(&p.key) && !self.block_cache.contains(&p.key)
        })
    }

    /// Overlay `id`'s tile over the base tile `base`'s cells: the level of
    /// its stack on the base tile's grid, `None` when its stack has none.
    fn plan_overlay_tile(
        &self,
        id: LayerId,
        base: &DetailTile,
        context: u64,
    ) -> Option<(PendingDetail, SliceRequest, StoreHandle)> {
        let plotted = self.plotted();
        let base_var = plotted.metadata.as_ref()?.variables.get(base.var_idx)?;
        let sel = self.layers.get(id)?.selection();
        let meta = sel.metadata.as_ref()?;
        let mut at_tile = plotted.clone();
        at_tile.variable_idx = base.var_idx;
        let idx = level_matching(&at_tile, meta, sel.variable_idx);
        let level = meta.variables.get(idx)?.multiscale.as_ref()?.level;
        let level0 = match at_base(sel) {
            Some(moved) => moved,
            None => sel.clone(),
        };
        let mut moved = match level {
            0 => level0,
            level => at_level(&level0, level)?,
        };
        let var = moved.variable_info()?.clone();
        let mut dims = [0; 2];
        for (i, &bd) in base.dims.iter().enumerate() {
            let name = base_var.dimension_names.get(bd)?;
            let d = var.dimension_names.iter().position(|n| n == name)?;
            // The same grid: the base level's length along the dimension.
            (var.shape.get(d) == base_var.shape.get(bd)).then_some(())?;
            *moved.dim_ranges.get_mut(d)? = (base.origin[i] as usize, base.end[i] as usize - 1);
            dims[i] = d;
        }
        let tile = DetailTile {
            level,
            var_idx: idx,
            dims,
            matrix: None,
            context,
            ..base.clone()
        };
        self.plan_tile(id, moved, tile)
    }
}

/// What identifies a base tile for its overlays: its plot, level and cells.
fn tile_key(t: &DetailTile) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (t.context, t.level, t.var_idx, t.origin, t.end).hash(&mut h);
    h.finish()
}
