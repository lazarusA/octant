//! Overlays' detail tiles follow the base layer's: as soon as the base
//! layer asks for a tile, each drawn multiscale overlay reads the level of
//! its own stack on that tile's grid over the same cells, drawn at the same
//! place on the quad.

use std::hash::{Hash, Hasher};

use super::want::plot_hash;
use super::{DetailTile, TilePlan};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::multiscale::{level_matching, to_level};

impl OctantApp {
    /// Keeps every overlay's tile on the base layer's latest one (pending,
    /// else shown): asks again when that tile or the overlay's selection
    /// changes; a tile stays drawn until its replacement arrives unless the
    /// overlay's selection changed, and every overlay tile goes once the
    /// base layer has none.
    pub(super) fn follow_detail_overlays(&mut self) {
        let target = self.base_target_tile().map(tile_key);
        for i in 0..self.layers.overlays().len() {
            let Some(id) = self.layers.overlays().get(i).map(|l| l.id()) else {
                break;
            };
            let own = self.overlay_context(id);
            let lost = self.layers.get(id).is_some_and(|l| self.fetch_lost(l));
            let Some(state) = self.layers.get_mut(id).map(|l| &mut l.data.detail) else {
                continue;
            };
            let (Some(own), Some(base_key)) = (own, target) else {
                state.clear();
                continue;
            };
            if lost {
                state.drop_request();
                state.follows = None;
            }
            if state.tile.as_ref().is_some_and(|t| t.context != own) {
                state.tile = None;
            }
            let follows = own ^ base_key.rotate_left(1);
            if state.follows == Some(follows) {
                continue;
            }
            state.follows = Some(follows);
            state.drop_request();
            if let Some(plan) = self.plan_overlay_tile(id, own) {
                self.request_tile(plan);
            }
        }
    }

    /// The base layer's tile being fetched, else shown.
    fn base_target_tile(&self) -> Option<&DetailTile> {
        let base = &self.layers.base.data.detail;
        base.pending
            .as_ref()
            .map(|p| &p.tile)
            .or(base.tile.as_ref())
    }

    /// What overlay `id`'s tile shows: its selection and step; `None` when
    /// it is not drawn.
    fn overlay_context(&self, id: LayerId) -> Option<u64> {
        let layer = self.layers.get(id).filter(|l| l.is_drawn())?;
        Some(plot_hash(
            layer.selection(),
            self.layer_step(id),
            &layer.data.flipped_dims,
            &layer.composite,
        ))
    }

    /// Overlay `id`'s tile over the base tile's cells: the level of its
    /// stack on the base tile's grid, `None` when its stack has none.
    fn plan_overlay_tile(&self, id: LayerId, context: u64) -> Option<TilePlan> {
        let base = self.base_target_tile()?;
        let base_var = self
            .plotted()
            .metadata
            .as_ref()?
            .variables
            .get(base.var_idx)?;
        let sel = self.layers.get(id)?.selection();
        let meta = sel.metadata.as_ref()?;
        let idx = level_matching(Some(base_var), meta, sel.variable_idx);
        let level = meta.variables.get(idx)?.multiscale.as_ref()?.level;
        let mut moved = to_level(sel, level)?;
        let var = moved.variable_info()?;
        let mut dims = [0; 2];
        for (i, &bd) in base.dims.iter().enumerate() {
            let name = base_var.dimension_names.get(bd)?;
            let d = var.dimension_names.iter().position(|n| n == name)?;
            // The same grid: the base level's length along the dimension.
            (var.shape.get(d) == base_var.shape.get(bd)).then_some(())?;
            dims[i] = d;
        }
        for (i, &d) in dims.iter().enumerate() {
            *moved.dim_ranges.get_mut(d)? = (base.origin[i] as usize, base.end[i] as usize - 1);
        }
        let tile = DetailTile {
            level,
            var_idx: idx,
            dims,
            matrix: None,
            probe: None,
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
