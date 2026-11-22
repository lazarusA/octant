//! The prefetcher's pending requests: starting one, cancelling one, and
//! which timesteps they already cover.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::super::cache::BlockCacheKey;
use super::super::prefetch_task::Task;
use super::BlockPrefetcher;
use crate::data::slice_request::DimensionSelection;

impl BlockPrefetcher {
    /// Counts `key` pending, on a worker, with a new request id and cancel
    /// flag: its task.
    pub(super) fn track(&mut self, key: BlockCacheKey) -> Task {
        self.pending.insert(key.clone());
        self.active_worker_threads.fetch_add(1, Ordering::Relaxed);
        self.last_request += 1;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancels
            .insert(key.clone(), (self.last_request, cancel.clone()));
        Task {
            key,
            request: self.last_request,
            tx: self.tx.clone(),
            aborted: self.aborted.clone(),
            cancel,
            active: self.active_worker_threads.clone(),
            freed: self.freed.clone(),
        }
    }

    /// Drops `key`'s request: queued, it never starts; running, it ends
    /// into the result channel unasked (the caller caches it). Whether it
    /// was pending.
    pub fn cancel(&mut self, key: &BlockCacheKey) -> bool {
        if let Some((_, flag)) = self.cancels.remove(key) {
            flag.store(true, Ordering::Relaxed);
        }
        self.pending.remove(key)
    }

    /// Checks if any pending in-flight request already covers the given timestep and matching spatial selections.
    pub fn is_pending_timestep(
        &self,
        source_id: &str,
        variable_name: &str,
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> bool {
        self.pending.iter().any(|key| {
            if key.source_id != source_id || key.variable_name != variable_name {
                return false;
            }
            let non_anim_matches = selections
                .iter()
                .enumerate()
                .filter(|&(d, _)| Some(d) != anim_dim)
                .all(|(d, sel)| key.selections.get(d) == Some(sel));
            if !non_anim_matches {
                return false;
            }
            anim_dim
                .and_then(|d| key.selections.get(d))
                .is_none_or(|sel| {
                    let (start, end) = sel.bounds();
                    timestep >= start && timestep < end
                })
        })
    }
}
