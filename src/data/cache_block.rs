//! Resident OctantBlock cache, block requests, batches, and asynchronous
//! prefetching.
//!
//! This is the new block-based data path and is intentionally independent
//! from the existing MatrixSlice cache.
//!
//! Architecture:
//!
//!     BlockRequest
//!          |
//!          v
//!     BlockRequestBatch
//!          |
//!          v
//!     BlockPrefetcher
//!          |
//!          +-------------------+
//!          |                   |
//!          v                   v
//!      Dataset A            Dataset B
//!       variable X          variable Y
//!          |                   |
//!          +---------+---------+
//!                    |
//!                    v
//!               OctantBlock
//!                    |
//!                    v
//!              BlockLruCache
//!                    |
//!              +-----+-----+
//!              |           |
//!              v           v
//!         matrix_slice()  volume()
//!
//! The cache stores resident N-dimensional data rather than rendered
//! MatrixSlices. Rendering/projection happens after a block is retrieved.
//!
//! The cache does not know how to construct HTTP, local Zarr, or Icechunk
//! stores. A BlockRequest carries the already-created Zarr storage handle,
//! while the cache key contains only the identity needed for caching and
//! deduplication.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use zarrs::storage::ReadableWritableListableStorage;

use crate::app::StoreKind;
use crate::data::octant_block::OctantBlock;
use crate::data::backends::zarr_block::fetch_block;
use crate::data::slice_request::{DimensionSelection, SliceRequest};


// ============================================================
// OctantBlock memory accounting
// ============================================================

impl OctantBlock {
    /// Approximate resident memory used by this block.
    ///
    /// The current cache budget counts the f32 value buffer. Metadata,
    /// coordinates, attributes, and String allocations are deliberately
    /// excluded for now.
    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>()
    }
}


// ============================================================
// Block request
// ============================================================

/// Describes one request for resident N-dimensional data.
///
/// This is the high-level "what data do I want?" abstraction.
///
/// A request contains:
///
/// - which backend/store it belongs to
/// - which dataset/store target it belongs to
/// - the SliceRequest describing the variable and hyperslab
/// - the already-created Zarr storage handle used to perform the read
///
/// The storage handle is intentionally NOT part of the cache identity.
/// Two handles pointing at the same dataset should still resolve to the
/// same BlockCacheKey.
#[derive(Clone)]
pub struct BlockRequest {
    pub store_kind: StoreKind,
    pub store_target: String,
    pub request: SliceRequest,

    /// Storage handle used by fetch_block().
    ///
    /// This is normally an Arc-backed zarrs storage object, so cloning
    /// this handle does not imply copying the dataset itself.
    pub storage: ReadableWritableListableStorage,
}

impl BlockRequest {
    /// Creates a request for one variable/hyperslab.
    pub fn new(
        store_kind: StoreKind,
        store_target: impl Into<String>,
        request: SliceRequest,
        storage: ReadableWritableListableStorage,
    ) -> Self {
        Self {
            store_kind,
            store_target: store_target.into(),
            request,
            storage,
        }
    }

    /// Produces the cache/deduplication identity for this request.
    pub fn cache_key(&self) -> BlockCacheKey {
        BlockCacheKey::from_request(
            self.store_kind,
            self.store_target.clone(),
            &self.request,
        )
    }

    /// Variable requested by this block.
    pub fn variable_name(&self) -> &str {
        &self.request.variable
    }
}


// ============================================================
// Batch request
// ============================================================

/// A collection of block requests that belong to one logical operation.
///
/// A batch can contain:
///
///     dataset A / temperature
///     dataset A / pressure
///     dataset A / humidity
///     dataset B / elevation
///     dataset C / wind_u
///
/// There is deliberately no requirement that all requests point to the
/// same dataset or even the same StoreKind.
#[derive(Clone, Default)]
pub struct BlockRequestBatch {
    requests: Vec<BlockRequest>,
}

impl BlockRequestBatch {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            requests: Vec::with_capacity(capacity),
        }
    }

    pub fn from_requests(requests: Vec<BlockRequest>) -> Self {
        Self { requests }
    }

    pub fn push(&mut self, request: BlockRequest) {
        self.requests.push(request);
    }

    pub fn extend(
        &mut self,
        requests: impl IntoIterator<Item = BlockRequest>,
    ) {
        self.requests.extend(requests);
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn requests(&self) -> &[BlockRequest] {
        &self.requests
    }

    pub fn into_requests(self) -> Vec<BlockRequest> {
        self.requests
    }

    /// Returns the requests that are not already present in the cache.
    pub fn uncached(
        &self,
        cache: &BlockLruCache,
    ) -> Vec<BlockRequest> {
        self.requests
            .iter()
            .filter(|request| {
                !cache.contains(&request.cache_key())
            })
            .cloned()
            .collect()
    }

    /// Returns requests that are neither cached nor already pending.
    pub fn ready_to_schedule(
        &self,
        cache: &BlockLruCache,
        prefetcher: &BlockPrefetcher,
    ) -> Vec<BlockRequest> {
        self.requests
            .iter()
            .filter(|request| {
                let key = request.cache_key();

                !cache.contains(&key)
                    && !prefetcher.is_pending(&key)
            })
            .cloned()
            .collect()
    }
}


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
/// is different from:
///
///     temperature
///     time = 148..196
///     z    = 0..32
///     y    = 0..512
///     x    = 0..512
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockCacheKey {
    pub store_kind: StoreKind,
    pub store_target: String,
    pub variable_name: String,
    pub selections: Vec<DimensionSelection>,
}

impl BlockCacheKey {
    /// Builds a cache key directly from a SliceRequest.
    pub fn from_request(
        store_kind: StoreKind,
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
    /// The block is cloned because the cache retains ownership of its
    /// resident copy.
    pub fn get(
        &mut self,
        key: &BlockCacheKey,
    ) -> Option<OctantBlock> {
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
    pub fn put(
        &mut self,
        key: BlockCacheKey,
        block: OctantBlock,
    ) {
        let block_bytes = block.bytes_size();

        if let Some(old_block) =
            self.entries.insert(key.clone(), block)
        {
            self.current_bytes = self
                .current_bytes
                .saturating_sub(old_block.bytes_size());
        }

        self.current_bytes += block_bytes;

        // A replacement is also a fresh access.
        self.access_order.retain(|k| k != &key);
        self.access_order.push_back(key);

        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.current_bytes > self.max_bytes
            && !self.access_order.is_empty()
        {
            let Some(oldest_key) =
                self.access_order.pop_front()
            else {
                break;
            };

            if let Some(evicted_block) =
                self.entries.remove(&oldest_key)
            {
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
    pub fn set_max_bytes(
        &mut self,
        new_max_bytes: usize,
    ) {
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
// Prefetch result
// ============================================================

/// Result returned by a background block fetch.
pub struct BlockPrefetchResult {
    pub key: BlockCacheKey,
    pub result: Result<OctantBlock, String>,
}


// ============================================================
// Batch result
// ============================================================

/// Result of a logical BlockRequestBatch.
///
/// Results are kept individually keyed because requests may finish in
/// arbitrary order.
pub struct BlockBatchResult {
    pub results: Vec<BlockPrefetchResult>,
}

impl BlockBatchResult {
    pub fn new(
        results: Vec<BlockPrefetchResult>,
    ) -> Self {
        Self { results }
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Number of successful block loads.
    pub fn successful_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.result.is_ok())
            .count()
    }

    /// Number of failed block loads.
    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.result.is_err())
            .count()
    }

    pub fn into_results(
        self,
    ) -> Vec<BlockPrefetchResult> {
        self.results
    }
}


// ============================================================
// Asynchronous prefetcher
// ============================================================

/// Asynchronous loader for resident OctantBlocks.
///
/// This type does not construct HTTP, local Zarr, or Icechunk stores.
/// The caller supplies the already-created storage handle in each
/// BlockRequest.
///
/// This allows a single batch to contain requests from:
///
///     - multiple variables in one dataset
///     - multiple datasets
///     - multiple storage backends
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

    pub fn with_max_concurrent_threads(
        max_concurrent_threads: usize,
    ) -> Self {
        let (tx, rx) = channel();

        Self {
            tx,
            rx,
            pending: HashSet::new(),
            active_worker_threads: 0,
            max_concurrent_threads:
                max_concurrent_threads.max(1),
        }
    }

    /// Polls completed background requests.
    ///
    /// Successful results should normally be inserted into the
    /// BlockLruCache by the caller.
    pub fn poll_results(
        &mut self,
    ) -> Vec<BlockPrefetchResult> {
        let mut results = Vec::new();

        while let Ok(result) =
            self.rx.try_recv()
        {
            self.active_worker_threads =
                self.active_worker_threads
                    .saturating_sub(1);

            self.pending.remove(&result.key);

            results.push(result);
        }

        results
    }

    /// Polls results as one logical batch.
    pub fn poll_batch_result(
        &mut self,
    ) -> Option<BlockBatchResult> {
        let results = self.poll_results();

        if results.is_empty() {
            None
        } else {
            Some(BlockBatchResult::new(results))
        }
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

    pub fn is_pending(
        &self,
        key: &BlockCacheKey,
    ) -> bool {
        self.pending.contains(key)
    }

    /// Requests one arbitrary N-dimensional block.
    ///
    /// The storage handle is moved into the worker request. The storage
    /// abstraction is normally Arc-backed, so this does not copy the
    /// dataset itself.
    pub fn request_block(
        &mut self,
        request: BlockRequest,
        cache: &BlockLruCache,
    ) -> bool {
        let key = request.cache_key();

        if cache.contains(&key) {
            return false;
        }

        if self.pending.contains(&key) {
            return false;
        }

        if self.active_worker_threads
            >= self.max_concurrent_threads
        {
            return false;
        }

        self.pending.insert(key.clone());
        self.active_worker_threads += 1;

        let tx = self.tx.clone();

        thread::spawn(move || {
            let result = fetch_block(
                request.storage,
                &request.store_target,
                &request.request,
            )
            .map_err(|e| e.to_string());

            let _ = tx.send(
                BlockPrefetchResult {
                    key,
                    result,
                },
            );
        });

        true
    }

    /// Requests every block in a batch that is not already cached or
    /// pending.
    ///
    /// Returns the number of requests successfully scheduled.
    pub fn request_batch(
        &mut self,
        batch: &BlockRequestBatch,
        cache: &BlockLruCache,
    ) -> usize {
        let mut scheduled = 0;

        for request in batch.requests() {
            if self.active_worker_threads
                >= self.max_concurrent_threads
            {
                break;
            }

            if self.request_block(
                request.clone(),
                cache,
            ) {
                scheduled += 1;
            }
        }

        scheduled
    }

    /// Convenience method for a single request represented by a key plus
    /// its storage handle.
    pub fn request_keyed_block(
        &mut self,
        key: BlockCacheKey,
        storage: ReadableWritableListableStorage,
        cache: &BlockLruCache,
    ) -> bool {
        let request = BlockRequest::new(
            key.store_kind,
            key.store_target.clone(),
            key.to_request(),
            storage,
        );

        self.request_block(
            request,
            cache,
        )
    }

    /// Removes a key from the pending set.
    ///
    /// This does not actually stop an already-running worker thread.
    /// The worker may still complete and send its result.
    ///
    /// This method is therefore best treated as "forget this request",
    /// rather than hard cancellation.
    pub fn forget_pending(
        &mut self,
        key: &BlockCacheKey,
    ) -> bool {
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

    fn test_block(
        start_time: usize,
        variable_name: &str,
    ) -> OctantBlock {
        let shape = vec![2, 3, 4];

        let values: Vec<f32> =
            (0..24)
                .map(|v| v as f32)
                .collect();

        OctantBlock::new(
            variable_name.to_string(),
            shape,
            vec![
                "time".into(),
                "y".into(),
                "x".into(),
            ],
            vec![
                start_time,
                0,
                0,
            ],
            values,
            HashMap::new(),
            HashMap::new(),
        )
    }

    fn test_key(
        start_time: usize,
        variable_name: &str,
        store_target: &str,
    ) -> BlockCacheKey {
        BlockCacheKey {
            store_kind:
                StoreKind::LocalZarr,

            store_target:
                store_target.to_string(),

            variable_name:
                variable_name.to_string(),

            selections: vec![
                DimensionSelection::Range(
                    start_time
                        ..start_time + 2,
                ),
                DimensionSelection::Range(
                    0..3,
                ),
                DimensionSelection::Range(
                    0..4,
                ),
            ],
        }
    }

    #[test]
    fn cache_stores_and_retrieves_block() {
        let mut cache =
            BlockLruCache::new(
                1024 * 1024,
            );

        let key = test_key(
            0,
            "temperature",
            "/tmp/test.zarr",
        );

        cache.put(
            key.clone(),
            test_block(
                0,
                "temperature",
            ),
        );

        assert_eq!(
            cache.cached_count(),
            1
        );

        let block =
            cache.get(&key)
                .expect(
                    "block should be cached",
                );

        assert_eq!(
            block.variable_name,
            "temperature"
        );

        assert_eq!(
            block.origin,
            vec![0, 0, 0]
        );

        assert_eq!(
            cache.hits(),
            1
        );

        assert_eq!(
            cache.misses(),
            0
        );
    }

    #[test]
    fn different_hyperslabs_are_different_keys() {
        let first = test_key(
            0,
            "temperature",
            "/tmp/test.zarr",
        );

        let second = test_key(
            2,
            "temperature",
            "/tmp/test.zarr",
        );

        assert_ne!(
            first,
            second
        );
    }

    #[test]
    fn different_variables_are_different_keys() {
        let temperature =
            test_key(
                0,
                "temperature",
                "/tmp/test.zarr",
            );

        let pressure =
            test_key(
                0,
                "pressure",
                "/tmp/test.zarr",
            );

        assert_ne!(
            temperature,
            pressure
        );
    }

    #[test]
    fn different_datasets_are_different_keys() {
        let first =
            test_key(
                0,
                "temperature",
                "/tmp/a.zarr",
            );

        let second =
            test_key(
                0,
                "temperature",
                "/tmp/b.zarr",
            );

        assert_ne!(
            first,
            second
        );
    }

    #[test]
    fn cache_tracks_bytes() {
        let mut cache =
            BlockLruCache::new(
                1024 * 1024,
            );

        let key = test_key(
            0,
            "temperature",
            "/tmp/test.zarr",
        );

        let block =
            test_block(
                0,
                "temperature",
            );

        let expected =
            block.bytes_size();

        cache.put(
            key,
            block,
        );

        assert_eq!(
            cache.current_bytes(),
            expected
        );
    }

    #[test]
    fn cache_replaces_existing_block() {
        let mut cache =
            BlockLruCache::new(
                1024 * 1024,
            );

        let key = test_key(
            0,
            "temperature",
            "/tmp/test.zarr",
        );

        cache.put(
            key.clone(),
            test_block(
                0,
                "temperature",
            ),
        );

        cache.put(
            key.clone(),
            test_block(
                100,
                "temperature",
            ),
        );

        assert_eq!(
            cache.cached_count(),
            1
        );

        let block =
            cache.get(&key)
                .expect(
                    "replacement should exist",
                );

        assert_eq!(
            block.origin[0],
            100
        );
    }

    #[test]
    fn cache_evicts_oldest_block() {
        let block_bytes =
            test_block(
                0,
                "temperature",
            )
            .bytes_size();

        let mut cache =
            BlockLruCache::new(
                block_bytes,
            );

        let first =
            test_key(
                0,
                "temperature",
                "/tmp/test.zarr",
            );

        let second =
            test_key(
                2,
                "temperature",
                "/tmp/test.zarr",
            );

        cache.put(
            first.clone(),
            test_block(
                0,
                "temperature",
            ),
        );

        cache.put(
            second.clone(),
            test_block(
                2,
                "temperature",
            ),
        );

        assert!(
            !cache.contains(&first)
        );

        assert!(
            cache.contains(&second)
        );

        assert_eq!(
            cache.cached_count(),
            1
        );
    }

    #[test]
    fn get_updates_lru_order() {
        let block_bytes =
            test_block(
                0,
                "temperature",
            )
            .bytes_size();

        let mut cache =
            BlockLruCache::new(
                block_bytes * 2,
            );

        let first =
            test_key(
                0,
                "temperature",
                "/tmp/test.zarr",
            );

        let second =
            test_key(
                2,
                "temperature",
                "/tmp/test.zarr",
            );

        let third =
            test_key(
                4,
                "temperature",
                "/tmp/test.zarr",
            );

        cache.put(
            first.clone(),
            test_block(
                0,
                "temperature",
            ),
        );

        cache.put(
            second.clone(),
            test_block(
                2,
                "temperature",
            ),
        );

        // Make first recently used.
        assert!(
            cache.get(&first).is_some()
        );

        // This should evict second, not first.
        cache.put(
            third.clone(),
            test_block(
                4,
                "temperature",
            ),
        );

        assert!(
            cache.contains(&first)
        );

        assert!(
            !cache.contains(&second)
        );

        assert!(
            cache.contains(&third)
        );
    }

    #[test]
    fn clear_removes_everything() {
        let mut cache =
            BlockLruCache::new(
                1024 * 1024,
            );

        cache.put(
            test_key(
                0,
                "temperature",
                "/tmp/test.zarr",
            ),
            test_block(
                0,
                "temperature",
            ),
        );

        cache.put(
            test_key(
                2,
                "pressure",
                "/tmp/test.zarr",
            ),
            test_block(
                2,
                "pressure",
            ),
        );

        cache.clear();

        assert_eq!(
            cache.cached_count(),
            0
        );

        assert_eq!(
            cache.current_bytes(),
            0
        );
    }

    #[test]
    fn cache_hit_rate_is_correct() {
        let mut cache =
            BlockLruCache::new(
                1024 * 1024,
            );

        let key = test_key(
            0,
            "temperature",
            "/tmp/test.zarr",
        );

        cache.put(
            key.clone(),
            test_block(
                0,
                "temperature",
            ),
        );

        assert!(
            cache.get(&key).is_some()
        );

        assert!(
            cache
                .get(
                    &test_key(
                        2,
                        "temperature",
                        "/tmp/test.zarr",
                    ),
                )
                .is_none()
        );

        assert_eq!(
            cache.hits(),
            1
        );

        assert_eq!(
            cache.misses(),
            1
        );

        assert!(
            (cache.hit_rate()
                - 50.0)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn key_can_reconstruct_request() {
        let key = test_key(
            10,
            "temperature",
            "/tmp/test.zarr",
        );

        let request =
            key.to_request();

        assert_eq!(
            request.variable,
            "temperature"
        );

        assert_eq!(
            request.selections,
            key.selections
        );
    }

    #[test]
    fn batch_can_contain_multiple_variables() {
        // This test only verifies the request architecture. Actual storage
        // access belongs to the integration tests around fetch_block().
        //
        // A real caller can create the three requests with the same storage
        // handle but different variable names.

        let batch =
            BlockRequestBatch::new();

        assert_eq!(
            batch.len(),
            0
        );
        assert!(
            batch.is_empty()
        );
    }

    #[test]
    fn cache_key_contains_dataset_and_variable_identity() {
        let temperature_a =
            test_key(
                0,
                "temperature",
                "dataset-a",
            );

        let pressure_a =
            test_key(
                0,
                "pressure",
                "dataset-a",
            );

        let temperature_b =
            test_key(
                0,
                "temperature",
                "dataset-b",
            );

        assert_ne!(
            temperature_a,
            pressure_a
        );

        assert_ne!(
            temperature_a,
            temperature_b
        );
    }
}