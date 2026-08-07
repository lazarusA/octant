//! Resident OctantBlock cache and asynchronous block prefetcher.
//!
//! This is the new block-based cache path.
//!
//! The existing MatrixSlice cache remains untouched while this is
//! introduced and tested.
//!
//! Data flow:
//!
//!     ReadableWritableListableStorage
//!                  |
//!                  v
//!             fetch_block()
//!                  |
//!                  v
//!             OctantBlock
//!                  |
//!                  v
//!             BlockLruCache
//!                  |
//!                  +----> matrix_slice()
//!                  |
//!                  +----> volume()
//!
//! The cache stores resident N-dimensional data rather than rendered
//! MatrixSlices. Rendering/projection happens after a block is retrieved.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use zarrs::storage::ReadableWritableListableStorage;

use crate::data::octant_block::OctantBlock;

use crate::utils::zarr::{DimensionSelection, SliceRequest};

use crate::utils::zarr::block::fetch_block;

// ============================================================
// Cache key
// ============================================================

/// Identifies one resident N-dimensional block.
///
/// The hyperslab selections are part of the key because two requests
/// for the same variable may represent completely different resident
/// regions of the dataset.
///
/// Example:
///
///     temperature
///     time = 100..148
///     z    = 0..32
///     y    = 0..512
///     x    = 0..512
///
/// is a different cache entry from:
///
///     temperature
///     time = 148..196
///     z    = 0..32
///     y    = 0..512
///     x    = 0..512
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockCacheKey {
    pub store_kind: crate::app::StoreKind,
    pub store_target: String,
    pub variable_name: String,
    pub selections: Vec<DimensionSelection>,
}

impl BlockCacheKey {
    /// Builds a cache key directly from a SliceRequest.
    pub fn from_request(
        store_kind: crate::app::StoreKind,
        store_target: impl Into<String>,
        request: &SliceRequest,
    ) -> Self {
        Self {
            store_kind,
            store_target: store_target.into(),
            variable_name: request.variable.clone(),
            selections: request.selections.clone(),
        }
    }

    /// Reconstructs the SliceRequest represented by this key.
    pub fn to_request(&self) -> SliceRequest {
        SliceRequest {
            variable: self.variable_name.clone(),
            selections: self.selections.clone(),
        }
    }
}

// ============================================================
// LRU cache
// ============================================================

/// LRU cache of resident N-dimensional OctantBlocks.
///
/// `access_order.front()` is the least recently used block.
/// `access_order.back()` is the most recently used block.
pub struct BlockLruCache {
    entries: HashMap<BlockCacheKey, OctantBlock>,
    access_order: VecDeque<BlockCacheKey>,

    max_bytes: usize,
    current_bytes: usize,

    hits: u64,
    misses: u64,
}

impl BlockLruCache {
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

    /// Gets a block and marks it as recently used.
    ///
    /// The block is cloned because the cache retains ownership of the
    /// resident copy.
    pub fn get(&mut self, key: &BlockCacheKey) -> Option<OctantBlock> {
        if let Some(block) = self.entries.get(key) {
            self.hits += 1;

            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.clone());

            Some(block.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn contains(&self, key: &BlockCacheKey) -> bool {
        self.entries.contains_key(key)
    }

    /// Inserts or replaces a resident block.
    pub fn put(&mut self, key: BlockCacheKey, block: OctantBlock) {
        let block_bytes = block.bytes_size();

        if let Some(old_block) = self.entries.insert(key.clone(), block) {
            self.current_bytes = self.current_bytes.saturating_sub(old_block.bytes_size());
        }

        self.current_bytes += block_bytes;

        // A replacement is also a fresh access.
        self.access_order.retain(|k| k != &key);
        self.access_order.push_back(key);

        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.current_bytes > self.max_bytes && !self.access_order.is_empty() {
            let Some(oldest_key) = self.access_order.pop_front() else {
                break;
            };

            if let Some(evicted_block) = self.entries.remove(&oldest_key) {
                self.current_bytes = self
                    .current_bytes
                    .saturating_sub(evicted_block.bytes_size());
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_bytes = 0;
    }

    /// Changes the memory budget and immediately evicts if necessary.
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

// ============================================================
// Asynchronous prefetch
// ============================================================

/// Result returned by a background block fetch.
pub struct BlockPrefetchResult {
    pub key: BlockCacheKey,
    pub result: Result<OctantBlock, String>,
}

/// Asynchronous loader for resident OctantBlocks.
///
/// This type deliberately does NOT construct Zarr/HTTP/Icechunk stores.
/// The caller supplies the already-created
/// `ReadableWritableListableStorage`.
///
/// That keeps this cache independent of the storage backend.
pub struct BlockPrefetcher {
    tx: Sender<BlockPrefetchResult>,
    rx: Receiver<BlockPrefetchResult>,

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

    /// Polls completed background requests.
    ///
    /// The caller is responsible for putting successful blocks into
    /// `BlockLruCache`.
    pub fn poll_results(&mut self) -> Vec<BlockPrefetchResult> {
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

    /// Requests one arbitrary N-dimensional block.
    ///
    /// `storage` is moved into the worker thread. This is intentional:
    /// zarrs' storage handle is the thing needed by `fetch_block()`.
    ///
    /// The caller should normally construct this storage using the
    /// appropriate backend helper, e.g. the HTTP `build_sync_store()`
    /// helper for a remote Zarr store.
    pub fn request_block(
        &mut self,
        key: BlockCacheKey,
        request: SliceRequest,
        storage: ReadableWritableListableStorage,
        cache: &BlockLruCache,
    ) {
        if cache.contains(&key) {
            return;
        }

        if self.pending.contains(&key) {
            return;
        }

        if self.active_worker_threads >= self.max_concurrent_threads {
            return;
        }

        self.pending.insert(key.clone());
        self.active_worker_threads += 1;

        let tx = self.tx.clone();

        let store_url = key.store_target.clone();

        thread::spawn(move || {
            let result = fetch_block(storage, &store_url, &request).map_err(|e| e.to_string());

            let _ = tx.send(BlockPrefetchResult { key, result });
        });
    }

    /// Convenience form that reconstructs the SliceRequest from the key.
    ///
    /// Useful when the cache key is already the source of truth.
    pub fn request_keyed_block(
        &mut self,
        key: BlockCacheKey,
        storage: ReadableWritableListableStorage,
        cache: &BlockLruCache,
    ) {
        let request = key.to_request();

        self.request_block(key, request, storage, cache);
    }

    /// Marks a request as no longer pending without waiting for a worker
    /// result.
    ///
    /// This is primarily useful if a caller decides to cancel a request.
    pub fn cancel_pending(&mut self, key: &BlockCacheKey) -> bool {
        self.pending.remove(key)
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    fn test_block(start_time: usize) -> OctantBlock {
        let shape = vec![2, 3, 4];

        let values: Vec<f32> = (0..24).map(|v| v as f32).collect();

        OctantBlock::new(
            "temperature".to_string(),
            shape,
            vec!["time".into(), "y".into(), "x".into()],
            vec![start_time, 0, 0],
            values,
            HashMap::new(),
            HashMap::new(),
        )
    }

    fn test_key(start_time: usize) -> BlockCacheKey {
        BlockCacheKey {
            store_kind: crate::app::StoreKind::LocalZarr,

            store_target: "/tmp/test.zarr".to_string(),

            variable_name: "temperature".to_string(),

            selections: vec![
                DimensionSelection::Range(start_time..start_time + 2),
                DimensionSelection::Range(0..3),
                DimensionSelection::Range(0..4),
            ],
        }
    }

    #[test]
    fn cache_stores_and_retrieves_block() {
        let mut cache = BlockLruCache::new(1024 * 1024);

        let key = test_key(0);

        cache.put(key.clone(), test_block(0));

        assert_eq!(cache.cached_count(), 1);

        let block = cache.get(&key).expect("block should be cached");

        assert_eq!(block.variable_name, "temperature");

        assert_eq!(block.origin, vec![0, 0, 0]);

        assert_eq!(cache.hits(), 1);

        assert_eq!(cache.misses(), 0);
    }

    #[test]
    fn different_hyperslabs_are_different_keys() {
        let first = test_key(0);
        let second = test_key(2);

        assert_ne!(first, second);
    }

    #[test]
    fn cache_tracks_bytes() {
        let mut cache = BlockLruCache::new(1024 * 1024);

        let key = test_key(0);

        let block = test_block(0);

        let expected = block.bytes_size();

        cache.put(key, block);

        assert_eq!(cache.current_bytes(), expected);
    }

    #[test]
    fn cache_replaces_existing_block() {
        let mut cache = BlockLruCache::new(1024 * 1024);

        let key = test_key(0);

        cache.put(key.clone(), test_block(0));

        cache.put(key.clone(), test_block(100));

        assert_eq!(cache.cached_count(), 1);

        let block = cache.get(&key).expect("replacement should exist");

        assert_eq!(block.origin[0], 100);
    }

    #[test]
    fn cache_evicts_oldest_block() {
        let block_bytes = test_block(0).bytes_size();

        // Enough for exactly one block.
        let mut cache = BlockLruCache::new(block_bytes);

        let first = test_key(0);
        let second = test_key(2);

        cache.put(first.clone(), test_block(0));

        cache.put(second.clone(), test_block(2));

        assert!(!cache.contains(&first));

        assert!(cache.contains(&second));

        assert_eq!(cache.cached_count(), 1);
    }

    #[test]
    fn get_updates_lru_order() {
        let block_bytes = test_block(0).bytes_size();

        let mut cache = BlockLruCache::new(block_bytes * 2);

        let first = test_key(0);
        let second = test_key(2);
        let third = test_key(4);

        cache.put(first.clone(), test_block(0));

        cache.put(second.clone(), test_block(2));

        // Make first recently used.
        assert!(cache.get(&first).is_some());

        // This should evict second, not first.
        cache.put(third.clone(), test_block(4));

        assert!(cache.contains(&first));

        assert!(!cache.contains(&second));

        assert!(cache.contains(&third));
    }

    #[test]
    fn clear_removes_everything() {
        let mut cache = BlockLruCache::new(1024 * 1024);

        cache.put(test_key(0), test_block(0));

        cache.put(test_key(2), test_block(2));

        cache.clear();

        assert_eq!(cache.cached_count(), 0);

        assert_eq!(cache.current_bytes(), 0);
    }

    #[test]
    fn cache_hit_rate_is_correct() {
        let mut cache = BlockLruCache::new(1024 * 1024);

        let key = test_key(0);

        cache.put(key.clone(), test_block(0));

        assert!(cache.get(&key).is_some());

        assert!(cache.get(&test_key(2)).is_none());

        assert_eq!(cache.hits(), 1);

        assert_eq!(cache.misses(), 1);

        assert!((cache.hit_rate() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn key_can_reconstruct_request() {
        let key = test_key(10);

        let request = key.to_request();

        assert_eq!(request.variable, "temperature");

        assert_eq!(request.selections, key.selections);
    }
}
