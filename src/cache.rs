use crate::app::StoreKind;
use crate::stores::{
    icechunk_local::IcechunkLocalStore, icechunk_remote::IcechunkRemoteStore,
    zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore, DataStore, MatrixSlice,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};
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
            if let Some(oldest_key) = self.access_order.pop_front() {
                if let Some(evicted_slice) = self.entries.remove(&oldest_key) {
                    self.current_bytes = self.current_bytes.saturating_sub(evicted_slice.bytes_size());
                }
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

pub struct SlicePrefetcher {
    tx: Sender<PrefetchResult>,
    rx: Receiver<PrefetchResult>,
    pending: HashSet<SliceCacheKey>,
}

impl SlicePrefetcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            pending: HashSet::new(),
        }
    }

    pub fn poll_results(&mut self) -> Vec<PrefetchResult> {
        let mut results = Vec::new();
        while let Ok(res) = self.rx.try_recv() {
            self.pending.remove(&res.key);
            results.push(res);
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

        let tx = self.tx.clone();
        let key_clone = key.clone();
        let store_target_for_thread = key.store_target.clone();
        let var_for_thread = key.variable_name.clone();

        thread::spawn(move || {
            let store: Box<dyn DataStore> = match key_clone.store_kind {
                StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&store_target_for_thread)),
                StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&store_target_for_thread)),
                StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&store_target_for_thread)),
                StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&store_target_for_thread)),
                StoreKind::ProceduralRandom => {
                    let _ = tx.send(PrefetchResult {
                        key: key_clone,
                        result: Err("Procedural random step".to_string()),
                    });
                    return;
                }
            };

            let res = store
                .fetch_slice(&var_for_thread, key_clone.timestep)
                .map_err(|e| e.to_string());

            let _ = tx.send(PrefetchResult {
                key: key_clone,
                result: res,
            });
        });
    }

    pub fn prefetch_chunk_aligned(
        &mut self,
        store_kind: StoreKind,
        store_target: &str,
        variable_name: &str,
        current_step: usize,
        max_steps: usize,
        chunk_time_size: usize,
        configured_lookahead: usize,
        cache: &SliceLruCache,
    ) {
        if max_steps <= 1 {
            return;
        }

        let effective_chunk_size = if chunk_time_size > 0 { chunk_time_size } else { 24 };
        let lookahead_count = effective_chunk_size.max(configured_lookahead);
        let store_target_string = store_target.to_string();

        for offset in 1..=lookahead_count {
            let step = (current_step + offset) % max_steps;
            let key = SliceCacheKey {
                store_kind,
                store_target: store_target_string.clone(),
                variable_name: variable_name.to_string(),
                timestep: step,
            };

            if cache.contains(&key) || self.pending.contains(&key) {
                continue;
            }

            self.pending.insert(key.clone());

            let tx = self.tx.clone();
            let key_clone = key.clone();
            let store_target_for_thread = store_target_string.clone();
            let var_for_thread = variable_name.to_string();

            thread::spawn(move || {
                let store: Box<dyn DataStore> = match key_clone.store_kind {
                    StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&store_target_for_thread)),
                    StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&store_target_for_thread)),
                    StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&store_target_for_thread)),
                    StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&store_target_for_thread)),
                    StoreKind::ProceduralRandom => {
                        let _ = tx.send(PrefetchResult {
                            key: key_clone,
                            result: Err("Procedural random step".to_string()),
                        });
                        return;
                    }
                };

                let res = store
                    .fetch_slice(&var_for_thread, key_clone.timestep)
                    .map_err(|e| e.to_string());

                let _ = tx.send(PrefetchResult {
                    key: key_clone,
                    result: res,
                });
            });
        }
    }
}
