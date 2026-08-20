//! Asynchronous block prefetching.
//!
//! Operates purely through `BlockRequest`, which carries its own store, so
//! this is backend-agnostic and works across requests that target
//! different `StoreHandle`s within the same batch.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;

use super::{
    block_cache::{BlockCache, BlockCacheKey},
    block_loader::BlockLoader,
    block_request::{BlockRequest, BlockRequestBatch},
    octant_block::OctantBlock,
};

pub struct PrefetchResult {
    pub key: BlockCacheKey,
    pub result: Result<OctantBlock, String>,
}

pub struct BlockPrefetcher {
    tx: SyncSender<PrefetchResult>,
    rx: Receiver<PrefetchResult>,
    pending: HashMap<BlockCacheKey, u64>,
    active_worker_threads: usize,
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
        let max_threads = max_concurrent_threads.max(1);
        let (tx, rx) = sync_channel(max_threads * 2);

        Self {
            tx,
            rx,
            pending: HashMap::new(),
            active_worker_threads: 0,
            max_concurrent_threads: max_threads,
            completed_bytes: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Schedules one request.
    pub fn request(&mut self, request: BlockRequest, cache: &BlockCache) -> bool {
        let key = request.cache_key();

        if cache.contains(&key) {
            return false;
        }

        if self.pending.contains_key(&key) {
            return false;
        }

        if self.active_worker_threads >= self.max_concurrent_threads {
            return false;
        }

        let estimated_bytes = (request.slice.estimated_elements() as u64) * 4;
        self.pending.insert(key.clone(), estimated_bytes);
        self.total_bytes
            .fetch_add(estimated_bytes, Ordering::Relaxed);
        self.active_worker_threads += 1;

        let tx = self.tx.clone();
        let completed_atomic = self.completed_bytes.clone();
        let aborted_atomic = self.aborted.clone();

        thread::spawn(move || {
            let mut on_progress = |chunk_bytes: u64| {
                if !aborted_atomic.load(Ordering::Relaxed) {
                    completed_atomic.fetch_add(chunk_bytes, Ordering::Relaxed);
                }
            };
            let result = BlockLoader::load_one_with_progress(&request, Some(&mut on_progress))
                .map_err(|error| error.to_string());
            let _ = tx.send(PrefetchResult { key, result });
        });

        true
    }

    /// Schedules every request in a batch that isn't already cached or pending.
    pub fn request_batch(&mut self, batch: &BlockRequestBatch, cache: &BlockCache) -> usize {
        let mut scheduled = 0;

        for request in batch.requests() {
            if self.active_worker_threads >= self.max_concurrent_threads {
                break;
            }

            if self.request(request.clone(), cache) {
                scheduled += 1;
            }
        }

        scheduled
    }

    pub fn poll(&mut self) -> Vec<PrefetchResult> {
        let mut results = Vec::new();

        while let Ok(result) = self.rx.try_recv() {
            self.active_worker_threads = self.active_worker_threads.saturating_sub(1);
            self.pending.remove(&result.key);
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
        let total = self.total_bytes.load(Ordering::Relaxed);
        if total > 0 {
            total
        } else {
            self.pending_bytes()
        }
    }

    pub fn active_worker_threads(&self) -> usize {
        self.active_worker_threads
    }

    pub fn max_concurrent_threads(&self) -> usize {
        self.max_concurrent_threads
    }

    pub fn is_pending(&self, key: &BlockCacheKey) -> bool {
        self.pending.contains_key(key)
    }

    /// Checks if any pending in-flight request already covers the given timestep.
    pub fn is_pending_timestep(
        &self,
        source_id: &str,
        variable_name: &str,
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> bool {
        self.pending.keys().any(|key| {
            if key.source_id == source_id && key.variable_name == variable_name {
                if let Some(dim) = anim_dim {
                    if let Some(sel) = key.selections.get(dim) {
                        let (start, end) = sel.bounds();
                        timestep >= start && timestep < end
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                false
            }
        })
    }

    pub fn forget_pending(&mut self, key: &BlockCacheKey) -> bool {
        self.pending.remove(key).is_some()
    }

    /// Cancels all currently queued/in-flight prefetch requests.
    pub fn abort(&mut self) {
        self.aborted.store(true, Ordering::Relaxed);
        self.aborted = Arc::new(AtomicBool::new(false));
        let (tx, rx) = sync_channel(self.max_concurrent_threads * 2);
        self.tx = tx;
        self.rx = rx;
        self.pending.clear();
        self.completed_bytes.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.active_worker_threads = 0;
    }
}
