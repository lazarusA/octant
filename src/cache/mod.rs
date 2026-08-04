use crate::app::StoreKind;
use crate::stores::{
    DataStore, MatrixSlice, icechunk_local::IcechunkLocalStore,
    icechunk_remote::IcechunkRemoteStore, zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SliceCacheKey {
    pub store_kind: StoreKind,
    pub store_target: String,
    pub variable_name: String,
    pub timestep: usize,
}

pub struct SliceLruCache {
    entries: HashMap<SliceCacheKey, MatrixSlice>,
    access_order: VecDeque<SliceCacheKey>,
    max_bytes: usize,
    current_bytes: usize,
    hits: u64,
    misses: u64,
}

impl SliceLruCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_bytes,
            current_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &SliceCacheKey) -> Option<MatrixSlice> {
        if let Some(slice) = self.entries.get(key) {
            self.hits += 1;
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());
            Some(slice.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn contains(&self, key: &SliceCacheKey) -> bool {
        self.entries.contains_key(key)
    }

    pub fn put(&mut self, key: SliceCacheKey, slice: MatrixSlice) {
        let slice_bytes = slice.bytes_size();

        if let Some(old_slice) = self.entries.insert(key.clone(), slice) {
            self.current_bytes = self.current_bytes.saturating_sub(old_slice.bytes_size());
            self.current_bytes += slice_bytes;
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key);
        } else {
            self.access_order.push_back(key);
            self.current_bytes += slice_bytes;
        }
        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.current_bytes > self.max_bytes && !self.access_order.is_empty() {
            if let Some(oldest_key) = self.access_order.pop_front()
                && let Some(evicted_slice) = self.entries.remove(&oldest_key)
            {
                self.current_bytes = self
                    .current_bytes
                    .saturating_sub(evicted_slice.bytes_size());
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_bytes = 0;
    }

    pub fn set_max_bytes(&mut self, new_max_bytes: usize) {
        self.max_bytes = new_max_bytes;
        self.evict_if_needed();
    }

    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn cached_count(&self) -> usize {
        self.entries.len()
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            100.0
        } else {
            (self.hits as f32 / total as f32) * 100.0
        }
    }
}

pub struct PrefetchResult {
    pub key: SliceCacheKey,
    pub result: Result<MatrixSlice, String>,
}

pub struct BatchResult {
    pub keys: Vec<SliceCacheKey>,
    pub results: Result<Vec<MatrixSlice>, String>,
}

pub struct SlicePrefetcher {
    tx: Sender<BatchResult>,
    rx: Receiver<BatchResult>,
    pending: HashSet<SliceCacheKey>,
    active_worker_threads: usize,
}

impl Default for SlicePrefetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SlicePrefetcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            pending: HashSet::new(),
            active_worker_threads: 0,
        }
    }

    pub fn poll_results(&mut self) -> Vec<PrefetchResult> {
        let mut results = Vec::new();
        while let Ok(batch_res) = self.rx.try_recv() {
            self.active_worker_threads = self.active_worker_threads.saturating_sub(1);
            for key in &batch_res.keys {
                self.pending.remove(key);
            }

            match batch_res.results {
                Ok(slices) => {
                    for slice in slices {
                        let key = SliceCacheKey {
                            store_kind: batch_res.keys[0].store_kind,
                            store_target: batch_res.keys[0].store_target.clone(),
                            variable_name: batch_res.keys[0].variable_name.clone(),
                            timestep: slice.current_timestep,
                        };
                        results.push(PrefetchResult {
                            key,
                            result: Ok(slice),
                        });
                    }
                }
                Err(err_msg) => {
                    for key in batch_res.keys {
                        results.push(PrefetchResult {
                            key,
                            result: Err(err_msg.clone()),
                        });
                    }
                }
            }
        }
        results
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn request_slice(&mut self, key: SliceCacheKey, cache: &SliceLruCache) {
        if cache.contains(&key) || self.pending.contains(&key) {
            return;
        }

        self.pending.insert(key.clone());
        self.active_worker_threads += 1;

        let tx = self.tx.clone();
        let key_clone = key.clone();
        let store_target_for_thread = key.store_target.clone();
        let var_for_thread = key.variable_name.clone();

        thread::spawn(move || {
            let store: Box<dyn DataStore> = match key_clone.store_kind {
                StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&store_target_for_thread)),
                StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&store_target_for_thread)),
                StoreKind::RemoteIcechunk => {
                    Box::new(IcechunkRemoteStore::new(&store_target_for_thread))
                }
                StoreKind::LocalIcechunk => {
                    Box::new(IcechunkLocalStore::new(&store_target_for_thread))
                }
                StoreKind::ProceduralRandom => {
                    let _ = tx.send(BatchResult {
                        keys: vec![key_clone],
                        results: Err("Procedural random step".to_string()),
                    });
                    return;
                }
            };

            let res = store
                .fetch_slice(&var_for_thread, key_clone.timestep)
                .map(|s| vec![s])
                .map_err(|e| e.to_string());

            let _ = tx.send(BatchResult {
                keys: vec![key_clone],
                results: res,
            });
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prefetch_chunk_aligned(
        &mut self,
        store_kind: StoreKind,
        store_target: &str,
        variable_name: &str,
        current_step: usize,
        max_steps: usize,
        chunk_time_size: usize,
        slice_bytes_hint: usize,
        configured_lookahead: usize,
        cache: &SliceLruCache,
    ) {
        if max_steps <= 1 {
            return;
        }

        let slice_bytes = if slice_bytes_hint > 0 {
            slice_bytes_hint
        } else {
            41_472
        };
        let cache_max_bytes = cache.max_bytes();
        let total_cacheable_slices = (cache_max_bytes / slice_bytes).max(1);
        let chunk_time_steps = if chunk_time_size > 0 {
            chunk_time_size
        } else {
            1
        };
        let target_prefetch_bytes = 50 * 1024 * 1024;
        let raw_target_slices = (target_prefetch_bytes / slice_bytes).max(1);

        let num_full_chunks = raw_target_slices / chunk_time_steps;
        let chunk_aligned_target = if num_full_chunks > 0 {
            num_full_chunks * chunk_time_steps
        } else {
            raw_target_slices
        };

        let max_safe_lookahead = (total_cacheable_slices * 2 / 3).max(1);

        let lookahead_count = if max_safe_lookahead >= chunk_time_steps {
            let chunks_in_memory = max_safe_lookahead / chunk_time_steps;
            let memory_aligned_limit = (chunks_in_memory * chunk_time_steps).max(chunk_time_steps);
            configured_lookahead
                .max(chunk_aligned_target)
                .min(memory_aligned_limit)
                .min(max_steps.saturating_sub(1))
                .max(1)
        } else {
            configured_lookahead
                .max(chunk_aligned_target)
                .min(max_safe_lookahead)
                .min(max_steps.saturating_sub(1))
                .max(1)
        };

        let max_concurrent_threads = 16;
        let store_target_string = store_target.to_string();
        let step_batch_size = (chunk_time_steps).max(1);
        let mut offset_count = 0;

        while offset_count < lookahead_count {
            if self.active_worker_threads >= max_concurrent_threads {
                break;
            }

            let start_step_in_batch = (current_step + 1 + offset_count) % max_steps;
            let remaining = lookahead_count - offset_count;
            let batch_count = step_batch_size.min(remaining).max(1);

            let batch_keys: Vec<SliceCacheKey> = (0..batch_count)
                .map(|i| SliceCacheKey {
                    store_kind,
                    store_target: store_target_string.clone(),
                    variable_name: variable_name.to_string(),
                    timestep: (start_step_in_batch + i) % max_steps,
                })
                .collect();

            let needs_fetch = batch_keys
                .iter()
                .any(|k| !cache.contains(k) && !self.pending.contains(k));

            if needs_fetch {
                for k in &batch_keys {
                    self.pending.insert(k.clone());
                }

                self.active_worker_threads += 1;

                let tx = self.tx.clone();
                let store_kind_for_thread = store_kind;
                let store_target_for_thread = store_target_string.clone();
                let var_for_thread = variable_name.to_string();
                let start_step_for_thread = start_step_in_batch;
                let fetch_count = batch_count;
                let thread_keys = batch_keys.clone();

                thread::spawn(move || {
                    let store: Box<dyn DataStore> = match store_kind_for_thread {
                        StoreKind::RemoteZarr => {
                            Box::new(ZarrRemoteStore::new(&store_target_for_thread))
                        }
                        StoreKind::LocalZarr => {
                            Box::new(ZarrLocalStore::new(&store_target_for_thread))
                        }
                        StoreKind::RemoteIcechunk => {
                            Box::new(IcechunkRemoteStore::new(&store_target_for_thread))
                        }
                        StoreKind::LocalIcechunk => {
                            Box::new(IcechunkLocalStore::new(&store_target_for_thread))
                        }
                        StoreKind::ProceduralRandom => {
                            let _ = tx.send(BatchResult {
                                keys: thread_keys,
                                results: Err("Procedural random step".to_string()),
                            });
                            return;
                        }
                    };

                    let res = store
                        .fetch_slice_range(&var_for_thread, start_step_for_thread, fetch_count)
                        .map_err(|e| e.to_string());

                    let _ = tx.send(BatchResult {
                        keys: thread_keys,
                        results: res,
                    });
                });
            }

            offset_count += batch_count;
        }
    }
}
