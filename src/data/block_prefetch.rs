//! Asynchronous block prefetching.
//!
//! Operates purely through `BlockRequest`, which carries its own store, so
//! this is backend-agnostic and works across requests that target
//! different `StoreHandle`s within the same batch.

use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};
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
    tx: Sender<PrefetchResult>,
    rx: Receiver<PrefetchResult>,
    pending: HashSet<BlockCacheKey>,
    active_worker_threads: usize,
    max_concurrent_threads: usize,
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
        let (tx, rx) = channel();

        Self {
            tx,
            rx,
            pending: HashSet::new(),
            active_worker_threads: 0,
            max_concurrent_threads: max_concurrent_threads.max(1),
        }
    }

    /// Schedules one request.
    pub fn request(&mut self, request: BlockRequest, cache: &BlockCache) -> bool {
        let key = request.cache_key();

        if cache.contains(&key) {
            return false;
        }

        if self.pending.contains(&key) {
            return false;
        }

        if self.active_worker_threads >= self.max_concurrent_threads {
            return false;
        }

        self.pending.insert(key.clone());
        self.active_worker_threads += 1;

        let tx = self.tx.clone();

        thread::spawn(move || {
            let result = BlockLoader::load_one(&request).map_err(|error| error.to_string());
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

        results
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn active_worker_threads(&self) -> usize {
        self.active_worker_threads
    }

    pub fn max_concurrent_threads(&self) -> usize {
        self.max_concurrent_threads
    }

    pub fn is_pending(&self, key: &BlockCacheKey) -> bool {
        self.pending.contains(key)
    }

    pub fn forget_pending(&mut self, key: &BlockCacheKey) -> bool {
        self.pending.remove(key)
    }
}
