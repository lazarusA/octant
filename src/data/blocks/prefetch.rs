//! Asynchronous block prefetching.
//!
//! Operates purely through `BlockRequest`, which carries its own store, so
//! this is backend-agnostic and works across requests that target
//! different `StoreHandle`s within the same batch.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use super::cache::{BlockCache, BlockCacheKey};
#[cfg(not(target_arch = "wasm32"))]
use super::loader::BlockLoader;
use super::request::BlockRequest;
use crate::data::{octant_block::OctantBlock, slice_request::DimensionSelection};

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
        let (tx, rx) = sync_channel(128);
        Self {
            tx,
            rx,
            pending: HashMap::new(),
            active_worker_threads: 0,
            max_concurrent_threads: max_concurrent_threads.clamp(1, 64),
            completed_bytes: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Schedules one request.
    pub fn request(&mut self, request: BlockRequest, cache: &BlockCache) -> bool {
        let key = request.cache_key();
        if cache.contains(&key)
            || self.pending.contains_key(&key)
            || self.active_worker_threads >= self.max_concurrent_threads
        {
            return false;
        }

        let estimated_bytes = (request.slice.estimated_elements() as u64) * 4;
        self.pending.insert(key.clone(), estimated_bytes);
        self.total_bytes
            .fetch_add(estimated_bytes, Ordering::Relaxed);
        self.active_worker_threads += 1;

        let (tx, completed, aborted) = (
            self.tx.clone(),
            self.completed_bytes.clone(),
            self.aborted.clone(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::TaskExecutor::spawn_background(move || {
            let mut on_progress = |chunk_bytes: u64| {
                if !aborted.load(Ordering::Relaxed) {
                    completed.fetch_add(chunk_bytes, Ordering::Relaxed);
                }
            };
            let result = crate::utils::executor::catch_panic(|| {
                BlockLoader::load_one_with_progress(&request, Some(&mut on_progress))
                    .map_err(|error| error.to_string())
            });
            let _ = tx.send(PrefetchResult { key, result });
        });

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let mut on_progress = |chunk_bytes: u64| {
                    if !aborted.load(Ordering::Relaxed) {
                        completed.fetch_add(chunk_bytes, Ordering::Relaxed);
                    }
                };
                let result = match request.store.source().kind {
                    crate::data::DataSourceKind::RemoteIcechunk
                    | crate::data::DataSourceKind::LocalIcechunk => {
                        crate::data::backends::icechunk::wasm::load_one_icechunk_wasm_with_progress(
                            &request,
                            Some(&mut on_progress),
                        )
                        .await
                    }
                    crate::data::DataSourceKind::RemoteGeoTiff
                    | crate::data::DataSourceKind::LocalGeoTiff => {
                        crate::data::backends::geotiff::wasm::load_one_geotiff_wasm_with_progress(
                            &request,
                            Some(&mut on_progress),
                        )
                        .await
                    }
                    crate::data::DataSourceKind::Procedural => request
                        .store
                        .fetch_with_progress(&request.slice, Some(&mut on_progress)),
                    _ => {
                        crate::data::backends::zarr::load_one_wasm_with_progress(
                            &request,
                            Some(&mut on_progress),
                        )
                        .await
                    }
                }
                .map_err(|error| error.to_string());
                let _ = tx.send(PrefetchResult { key, result });
            });
        }

        true
    }

    pub fn poll(&mut self) -> Vec<PrefetchResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.rx.try_recv() {
            self.active_worker_threads = self.active_worker_threads.saturating_sub(1);
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
        self.active_worker_threads
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

    /// Cancels all currently queued/in-flight prefetch requests.
    pub fn abort(&mut self) {
        self.aborted.store(true, Ordering::Relaxed);
        self.aborted = Arc::new(AtomicBool::new(false));
        let (tx, rx) = sync_channel(self.max_concurrent_threads * 2);
        (self.tx, self.rx) = (tx, rx);
        self.pending.clear();
        self.completed_bytes.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.active_worker_threads = 0;
    }
}
