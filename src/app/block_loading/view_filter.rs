//! Which resident blocks belong to the plotted view, and re-projecting cached
//! blocks after a new selection.

use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::octant_block::OctantBlock;
use crate::data::{BlockCacheKey, DimensionSelection};
use crate::plots::PlotType;

/// Whether a block spanning `origin..origin + len` overlaps the inclusive
/// requested range `req`.
pub fn overlaps(origin: usize, len: usize, req: (usize, usize)) -> bool {
    origin <= req.1 && origin + len > req.0
}

/// Whether `block` holds index `step` along dimension `dim` (none when the
/// block has no such dimension).
pub fn covers_step(block: &OctantBlock, dim: usize, step: usize) -> bool {
    let origin = block.origin.get(dim).copied().unwrap_or(0);
    let extent = block.shape.get(dim).copied().unwrap_or(0);
    (origin..origin + extent).contains(&step)
}

impl OctantApp {
    /// Whether `block` holds data of layer `id`'s current view: it overlaps the
    /// requested range on every spatial axis (`axes`: block dim and inclusive
    /// request), and covers the layer's step unless the animated axis is itself
    /// spatial (then the blocks along it together make up the volume).
    pub(crate) fn block_in_view(
        &self,
        id: LayerId,
        block: &OctantBlock,
        anim_dim: Option<usize>,
        anim_is_spatial: bool,
        axes: [(usize, (usize, usize)); 3],
    ) -> bool {
        let step = self.layer_step(id);
        let span = |dim: usize| {
            let origin = block.origin.get(dim).copied().unwrap_or(0);
            (origin, block.shape.get(dim).copied().unwrap_or(1))
        };
        if let Some(dim) = anim_dim.filter(|_| !anim_is_spatial)
            && dim < block.rank()
        {
            let (origin, len) = span(dim);
            if !(origin..origin + len).contains(&step) {
                return false;
            }
        }
        axes.iter()
            .filter(|(dim, _)| *dim < block.rank())
            .all(|&(dim, req)| {
                let (origin, len) = span(dim);
                if Some(dim) == anim_dim && anim_is_spatial {
                    overlaps(origin, len, req) || (origin..origin + len).contains(&step)
                } else {
                    overlaps(origin, len, req)
                }
            })
    }

    /// The layer whose latest requested view `block` (cache key `key`, not the
    /// request itself) belongs to: same variable and selection, and either the
    /// current step or a volume, which every block of the range builds.
    pub(crate) fn layer_showing_block(
        &self,
        key: &BlockCacheKey,
        block: &OctantBlock,
    ) -> Option<LayerId> {
        self.layers.iter().map(Layer::id).find(|&id| {
            let is_same_var = self
                .layer_variable_info(id)
                .is_some_and(|v| v.name == block.variable_name);
            let anim_dim = self.layer_animated_dim(id);
            if !is_same_var || !self.key_matches_view(id, key, anim_dim) {
                return false;
            }
            let covers_current =
                anim_dim.is_some_and(|dim| covers_step(block, dim, self.layer_step(id)));
            let is_volume = self.layer_selections(id).is_some_and(|(_, staged)| {
                matches!(staged.plot_type, PlotType::Volume | PlotType::PointCloud)
            });
            covers_current || is_volume
        })
    }

    /// Plots resident `block` in layer `id` for its selection of dataset
    /// `source_id` (`selections`, any step along `anim_dim`): syncs the base
    /// layer's plotted state, projects it (and the other blocks of a reset
    /// volume) and queues the rest of the animated range.
    pub(crate) fn show_cached_block(
        &mut self,
        id: LayerId,
        source_id: &str,
        block: &OctantBlock,
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
    ) {
        self.status_message = format!(
            "Block cache HIT for '{}' ({} bytes resident)",
            block.variable_name,
            block.bytes_size()
        );
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        layer.load.pending_target_step = None;
        if id == LayerId::BASE {
            self.sync_plotted_state_from_selected();
        }
        let since = self.layers.get(id).map_or(0, |l| l.data.volume_allocations);
        self.apply_block_projection(id, block);
        self.project_cached_volume_blocks(
            id,
            source_id,
            &block.variable_name,
            selections,
            anim_dim,
            since,
        );
        self.prefetch_layer_animated_range(id);
    }

    /// Projects every cached block of layer `id`'s selection after its volume
    /// was reset (`volume_allocations` moved past `since`, a new selection): a
    /// volume can be built from several resident blocks (along an animated
    /// spatial axis), and a cache hit only delivers the one covering the
    /// current step. Playback steps keep the volume and skip this.
    pub(crate) fn project_cached_volume_blocks(
        &mut self,
        id: LayerId,
        source_id: &str,
        var_name: &str,
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        since: u64,
    ) {
        let Some(layer) = self.layers.get(id) else {
            return;
        };
        if layer.data.volume_allocations == since {
            return;
        }
        let is_volume = self.layer_selections(id).is_some_and(|(_, staged)| {
            matches!(staged.plot_type, PlotType::Volume | PlotType::PointCloud)
        });
        if !is_volume {
            return;
        }
        let blocks = self
            .block_cache
            .matching_blocks(source_id, var_name, selections, anim_dim);
        for block in &blocks {
            self.apply_block_projection(id, block);
        }
    }
}
