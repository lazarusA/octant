//! What an arriving block settles: whether its layers' last fetch failed,
//! and for the block a layer requested, its plotted selection and step.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::OctantBlock;

impl OctantApp {
    /// Marks whether the last fetch of the layers reading blocks like `key`
    /// (its dataset and variable) failed; arrivals clear only failed layers.
    pub(super) fn mark_failed(&mut self, key: &crate::data::BlockCacheKey, failed: bool) {
        if !failed && !self.layers.iter().any(|l| l.load.failed) {
            return;
        }
        let ids: Vec<LayerId> = self.layers.iter().map(|l| l.id()).collect();
        for id in ids {
            let reads = self.layer_selections(id).is_some_and(|(shown, _)| {
                shown
                    .variable_info()
                    .is_some_and(|v| v.name == key.variable_name)
                    && self.selection_source_id(shown) == key.source_id
            });
            if let Some(layer) = self.layers.get_mut(id).filter(|_| reads) {
                layer.load.failed = failed;
            }
        }
    }

    /// Settles layer `id`'s request for `block`: the base layer's staged
    /// selection becomes the plotted one, and the view moves to the step the
    /// request was made at. Whether it moved to that step.
    pub(super) fn accept_requested_block(&mut self, id: LayerId, block: &OctantBlock) -> bool {
        if let Some(layer) = self.layers.get_mut(id) {
            layer.load.block_key = None;
        }
        if id == LayerId::BASE {
            self.sync_plotted_state_for(Some(&block.variable_name));
        }
        let Some(layer) = self.layers.get_mut(id) else {
            return false;
        };
        let Some(target) = layer.load.pending_target_step.take() else {
            return false;
        };
        let Some(dim) = layer.selection().animated_dim else {
            return false;
        };
        if !super::view_filter::covers_step(block, dim, target) {
            return false;
        }
        // Only the base layer moves the shared step; an overlay may have
        // clamped its own request into a shorter extent.
        if id == LayerId::BASE {
            self.playback.current_timestep = target;
        }
        let selection = if !layer.selection().dim_indices.is_empty() {
            Some(layer.selection_mut())
        } else {
            self.staged_selection_mut(id)
        };
        if let Some(index) = selection.and_then(|s| s.dim_indices.get_mut(dim)) {
            *index = target;
        }
        true
    }
}
