//! What one prefetch task shares with the [`BlockPrefetcher`](super::prefetch::BlockPrefetcher):
//! its result channel, the abort flag and its worker slot.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

use super::cache::BlockCacheKey;
use super::prefetch::PrefetchResult;
use crate::data::octant_block::OctantBlock;

pub(super) struct Task {
    pub(super) key: BlockCacheKey,
    pub(super) tx: Sender<PrefetchResult>,
    pub(super) aborted: Arc<AtomicBool>,
    pub(super) active: Arc<AtomicUsize>,
}

impl Task {
    /// Whether the prefetcher aborted while this task was queued: nobody
    /// reads its result, so it releases its worker slot and does nothing.
    pub(super) fn cancelled(&self) -> bool {
        let aborted = self.aborted.load(Ordering::Relaxed);
        if aborted {
            self.active.fetch_sub(1, Ordering::Relaxed);
        }
        aborted
    }

    /// Counts `bytes` downloaded, until an abort.
    pub(super) fn progress(&self, bytes: u64) {
        if !self.aborted.load(Ordering::Relaxed) {
            super::progress::bytes(bytes);
        }
    }

    /// Releases the worker slot and sends `result`.
    pub(super) fn finish(self, result: Result<OctantBlock, String>) {
        self.active.fetch_sub(1, Ordering::Relaxed);
        let _ = self.tx.send(PrefetchResult {
            key: self.key,
            result,
        });
    }
}
