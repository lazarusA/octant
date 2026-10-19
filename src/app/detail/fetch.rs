//! A detail tile's fetch: shown at once from the block cache, else asked of
//! the prefetcher (or kept until a worker frees up), and its result taken
//! when it arrives.

use super::{DetailState, TilePlan, Want};
use crate::app::OctantApp;
use crate::data::{BlockCacheKey, BlockRequest};

impl OctantApp {
    /// Asks for `want`'s tile: shown at once from the block cache, else
    /// fetched and shown when it arrives (`accept_detail_block`).
    pub(super) fn request_detail(&mut self, want: &Want) {
        if let Some(plan) = self.plan_detail(want) {
            self.request_tile(plan);
        }
    }

    /// Shows a planned tile at once from the block cache, else fetches it.
    pub(super) fn request_tile(&mut self, plan: TilePlan) {
        let (pending, slice, handle) = plan;
        if let Some(block) = self.block_cache.get(&pending.key) {
            self.show_detail(pending, &block);
            return;
        }
        let request = BlockRequest::new(handle.clone(), slice.clone());
        // Taken, or already fetched for another request (a layer's block).
        let taken = self.block_prefetcher.request(request, &self.block_cache)
            || self.block_prefetcher.is_pending(&pending.key);
        let Some(layer) = self.layers.get_mut(pending.layer) else {
            return;
        };
        let state = &mut layer.data.detail;
        state.drop_request();
        if taken {
            state.pending = Some(pending);
        } else {
            state.refused = Some((pending, slice, handle));
        }
    }

    /// Asks again for the tiles refused while every worker was busy; called
    /// once fetches complete.
    pub(crate) fn retry_refused_details(&mut self) {
        let plans: Vec<TilePlan> = self
            .layers
            .iter_mut()
            .filter_map(|l| l.data.detail.refused.take())
            .collect();
        for plan in plans {
            self.request_tile(plan);
        }
    }

    /// Takes a fetch result when it is the pending tile's (shown and cached,
    /// or dropped with a log when the fetch failed); whether it was.
    pub(crate) fn take_detail_result(
        &mut self,
        key: &BlockCacheKey,
        result: &Result<crate::data::OctantBlock, String>,
    ) -> bool {
        let waiting = |l: &crate::app::layers::Layer| {
            l.data
                .detail
                .pending
                .as_ref()
                .is_some_and(|p| &p.key == key)
        };
        let Some(id) = self.layers.iter().find(|l| waiting(l)).map(|l| l.id()) else {
            return false;
        };
        let Some(pending) = self
            .layers
            .get_mut(id)
            .and_then(|l| l.data.detail.pending.take())
        else {
            return false;
        };
        match result {
            Ok(block) => {
                self.block_cache.put(key.clone(), block.clone());
                self.show_detail(pending, block);
            }
            Err(e) => {
                log::warn!("Detail tile of '{}' failed: {e}", key.variable_name);
                if let Some(layer) = self.layers.get_mut(id) {
                    layer.data.detail.retry_failed();
                }
            }
        }
        true
    }
}

impl DetailState {
    /// After a failed fetch: decides the resting view again, once per view,
    /// so a tile that keeps failing is not asked for in a loop.
    pub(super) fn retry_failed(&mut self) {
        if self.failed != self.view {
            self.failed = self.view;
            self.view = None;
        }
    }
}
