//! Asynchronous block prefetching.
//!
//! Operates purely through `BlockRequest`, which carries its own store, so
//! this is backend-agnostic and works across requests that target
//! different `StoreHandle`s within the same batch.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;
use web_time::Instant;

use super::cache::{BlockCache, BlockCacheKey};
use super::loader::BlockLoader;
use super::prefetch_task::Task;
use super::request::BlockRequest;
use crate::data::{octant_block::OctantBlock, slice_request::DimensionSelection};

/// How long the lookahead leaves a failed block: asked every few seconds,
/// never in a loop, so playback recovers once the network does.
const RETRY_AFTER: Duration = Duration::from_secs(5);

pub struct PrefetchResult {
    pub key: BlockCacheKey,
    pub result: Result<OctantBlock, String>,
}

pub struct BlockPrefetcher {
    tx: SyncSender<PrefetchResult>,
    rx: Receiver<PrefetchResult>,
    pending: HashMap<BlockCacheKey, u64>,
    /// Blocks whose last fetch failed, and when: the lookahead skips them
    /// for `retry_after` (`failed_recently`), or until Play, Plot, Abort.
    failed: HashMap<BlockCacheKey, Instant>,
    pub(super) retry_after: Duration,
    /// Workers still running, aborted ones included: each task releases its
    /// own slot, so an abort never lets new fetches pile onto old ones.
    active_worker_threads: Arc<AtomicUsize>,
    max_concurrent_threads: usize,
    completed_bytes: Arc<AtomicU64>,
    total_bytes: Arc<AtomicU64>,
    aborted: Arc<AtomicBool>,
}

impl Default for BlockPrefetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockPrefetcher {
    pub fn new() -> Self {
        Self::with_max_concurrent_threads(16)
    }

    pub fn with_max_concurrent_threads(max_concurrent_threads: usize) -> Self {
        let (tx, rx) = sync_channel(128);
        Self {
            tx,
            rx,
            pending: HashMap::new(),
            failed: HashMap::new(),
            retry_after: RETRY_AFTER,
            active_worker_threads: Arc::new(AtomicUsize::new(0)),
            max_concurrent_threads: max_concurrent_threads.clamp(1, 64),
            completed_bytes: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Schedules one request, unless every worker is busy.
    pub fn request(&mut self, request: BlockRequest, cache: &BlockCache) -> bool {
        self.active_worker_threads() < self.max_concurrent_threads
            && self.request_now(request, cache)
    }

    /// Schedules one request even when every worker is busy: the block a view
    /// waits for never queues behind lookahead prefetches.
    pub fn request_now(&mut self, request: BlockRequest, cache: &BlockCache) -> bool {
        let key = request.cache_key();
        if cache.contains(&key) || self.pending.contains_key(&key) {
            return false;
        }

        let estimated_bytes = (request.slice.estimated_elements() as u64) * 4;
        self.pending.insert(key.clone(), estimated_bytes);
        self.total_bytes
            .fetch_add(estimated_bytes, Ordering::Relaxed);
        self.active_worker_threads.fetch_add(1, Ordering::Relaxed);

        let task = Task {
            key,
            tx: self.tx.clone(),
            completed: self.completed_bytes.clone(),
            aborted: self.aborted.clone(),
            active: self.active_worker_threads.clone(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::TaskExecutor::spawn_background(move || {
            if task.cancelled() {
                return;
            }
            let mut on_progress = |bytes| task.progress(bytes);
            let result = crate::utils::executor::catch_panic(|| {
                BlockLoader::load_one_with_progress(&request, Some(&mut on_progress))
                    .map_err(|error| error.to_string())
            });
            task.finish(result);
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if task.cancelled() {
                return;
            }
            let mut on_progress = |bytes| task.progress(bytes);
            let result = BlockLoader::load_one_wasm(&request, Some(&mut on_progress))
                .await
                .map_err(|error| error.to_string());
            task.finish(result);
        });

        true
    }

    pub fn poll(&mut self) -> Vec<PrefetchResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.rx.try_recv() {
            if result.result.is_err() {
                let retry_after = self.retry_after;
                self.failed.retain(|_, at| at.elapsed() < retry_after);
                self.failed.insert(result.key.clone(), Instant::now());
            } else {
                self.failed.remove(&result.key);
            }
            if let Some(est) = self.pending.remove(&result.key) {
                let _ = self
                    .total_bytes
                    .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                        Some(v.saturating_sub(est))
                    });
                let _ =
                    self.completed_bytes
                        .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                            Some(v.saturating_sub(est))
                        });
            }
            results.push(result);
        }

        if self.pending.is_empty() {
            self.completed_bytes.store(0, Ordering::Relaxed);
            self.total_bytes.store(0, Ordering::Relaxed);
        }
        results
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_bytes(&self) -> u64 {
        self.pending.values().copied().sum()
    }

    pub fn completed_bytes(&self) -> u64 {
        self.completed_bytes.load(Ordering::Relaxed)
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
            .load(Ordering::Relaxed)
            .max(self.pending_bytes())
    }

    pub fn active_worker_threads(&self) -> usize {
        self.active_worker_threads.load(Ordering::Relaxed)
    }

    /// Whether `key`'s last fetch failed less than `retry_after` ago.
    pub fn failed_recently(&self, key: &BlockCacheKey) -> bool {
        self.failed
            .get(key)
            .is_some_and(|at| at.elapsed() < self.retry_after)
    }

    /// Lets the lookahead try the blocks that failed again.
    pub fn clear_failed(&mut self) {
        self.failed.clear();
    }

    pub fn max_concurrent_threads(&self) -> usize {
        self.max_concurrent_threads
    }

    pub fn set_max_concurrent_threads(&mut self, max_threads: usize) {
        self.max_concurrent_threads = max_threads.clamp(1, 64);
    }

    pub fn is_pending(&self, key: &BlockCacheKey) -> bool {
        self.pending.contains_key(key)
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
        self.pending.keys().any(|key| {
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

    /// Drops every pending request: queued ones never start, running ones
    /// finish unread, holding their worker slots until then.
    pub fn abort(&mut self) {
        self.aborted.store(true, Ordering::Relaxed);
        self.aborted = Arc::new(AtomicBool::new(false));
        let (tx, rx) = sync_channel(self.max_concurrent_threads * 2);
        (self.tx, self.rx) = (tx, rx);
        self.pending.clear();
        self.completed_bytes.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.failed.clear();
    }
}
