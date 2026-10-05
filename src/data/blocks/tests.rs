//! Unit tests for block cache operations.

use crate::data::blocks::{BlockCache, BlockCacheKey};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use std::collections::HashMap as StdHashMap;

fn test_block(origin0: usize) -> OctantBlock {
    let shape = vec![2, 3, 4];
    let values: Vec<f32> = (0..24).map(|v| v as f32).collect();

    OctantBlock::new(
        "temperature".to_string(),
        shape,
        vec!["time".into(), "y".into(), "x".into()],
        vec![origin0, 0, 0],
        values,
        StdHashMap::new(),
        StdHashMap::new(),
    )
}

fn test_key(source_id: &str, variable: &str, start: usize) -> BlockCacheKey {
    let slice = SliceRequest::new(
        variable,
        vec![
            DimensionSelection::range(start, start + 2),
            DimensionSelection::range(0, 3),
            DimensionSelection::range(0, 4),
        ],
    );
    BlockCacheKey::new(source_id, &slice)
}

#[test]
fn test_block_cache_put_and_get() {
    let mut cache = BlockCache::new(1024 * 1024);
    let key = test_key("dataset-1", "temperature", 0);
    let block = test_block(0);

    cache.put(key.clone(), block);
    assert_eq!(cache.cached_count(), 1);
    assert!(cache.contains(&key));

    let retrieved = cache.get(&key);
    assert!(retrieved.is_some());
    let retrieved_block = retrieved.unwrap();
    assert_eq!(retrieved_block.variable_name, "temperature");
    assert_eq!(retrieved_block.origin, vec![0, 0, 0]);
}

#[test]
fn test_block_cache_lru_eviction() {
    // 24 elements * 4 bytes = 96 bytes per block
    // Capacity of 200 bytes holds at most 2 blocks (192 bytes)
    let mut cache = BlockCache::new(200);

    let key0 = test_key("dataset-1", "temperature", 0);
    let key1 = test_key("dataset-1", "temperature", 2);
    let key2 = test_key("dataset-1", "temperature", 4);

    cache.put(key0.clone(), test_block(0));
    cache.put(key1.clone(), test_block(2));
    assert_eq!(cache.cached_count(), 2);

    // Access key0 so key1 becomes the oldest
    let _ = cache.get(&key0);

    // Insert key2 -> should evict key1 (LRU)
    cache.put(key2.clone(), test_block(4));
    assert_eq!(cache.cached_count(), 2);
    assert!(cache.contains(&key0));
    assert!(!cache.contains(&key1));
    assert!(cache.contains(&key2));
}

#[test]
fn test_block_cache_hit_rate() {
    let mut cache = BlockCache::new(1024 * 1024);
    let key = test_key("dataset-1", "temperature", 0);

    // Miss
    let _ = cache.get(&key);
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 0);
    assert_eq!(cache.hit_rate(), 0.0);

    // Put + Hit
    cache.put(key.clone(), test_block(0));
    let _ = cache.get(&key);
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 1);
    assert_eq!(cache.hit_rate(), 50.0);
}

#[test]
fn test_block_cache_summary_and_clearing() {
    let mut cache = BlockCache::new(1024 * 1024);

    let key_temp_a = test_key("dataset-a", "temperature", 0);
    let key_salt_a = test_key("dataset-a", "salinity", 0);
    let key_temp_b = test_key("dataset-b", "temperature", 0);

    cache.put(key_temp_a, test_block(0));
    cache.put(key_salt_a, test_block(0));
    cache.put(key_temp_b, test_block(0));

    assert_eq!(cache.cached_count(), 3);
    let summaries = cache.summary_per_variable();
    assert_eq!(summaries.len(), 3);

    let var_bytes = cache.bytes_by_variable();
    assert_eq!(var_bytes.len(), 2);
    assert!(var_bytes.contains_key("temperature"));
    assert!(var_bytes.contains_key("salinity"));

    let source_bytes = cache.bytes_by_source();
    assert_eq!(source_bytes.len(), 2);
    assert!(source_bytes.contains_key("dataset-a"));
    assert!(source_bytes.contains_key("dataset-b"));

    // Clear only dataset-a salinity
    cache.clear_variable("dataset-a", "salinity");
    assert_eq!(cache.cached_count(), 2);
    assert_eq!(cache.summary_per_variable().len(), 2);

    // Clear dataset-a remaining
    cache.clear_source("dataset-a");
    assert_eq!(cache.cached_count(), 1);
    assert_eq!(cache.summary_per_variable()[0].source_id, "dataset-b");
}

#[test]
fn test_prefetcher_active_bytes_retire_on_drain() {
    use crate::data::blocks::{BlockPrefetcher, BlockStore, BlockStoreError, ProgressCallback};
    use crate::data::{DataSource, DataSourceKind, StoreHandle};
    use std::sync::Arc;

    struct DummyStore;
    impl BlockStore for DummyStore {
        fn backend_name(&self) -> &str {
            "dummy"
        }
        fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
            Ok(vec!["temperature".into()])
        }
        fn fetch_block_with_progress(
            &self,
            _request: &SliceRequest,
            _on_progress: ProgressCallback,
        ) -> Result<OctantBlock, BlockStoreError> {
            Ok(test_block(0))
        }
    }

    let mut prefetcher = BlockPrefetcher::new();
    let cache = BlockCache::new(1024 * 1024);
    let source = DataSource::new("dummy", DataSourceKind::Procedural, "dummy", "dummy");
    let store_handle = StoreHandle::new(source, Arc::new(DummyStore));

    let slice_1 = SliceRequest::new("temperature", vec![DimensionSelection::range(0, 10)]);
    let req_1 = crate::data::BlockRequest::new(store_handle, slice_1);

    assert!(prefetcher.request(req_1, &cache));
    assert_eq!(prefetcher.pending_count(), 1);
    assert_eq!(prefetcher.total_bytes(), 40); // 10 elements * 4 bytes
    assert_eq!(prefetcher.pending_bytes(), 40);

    // Draining without results still retains pending
    let results = prefetcher.poll();
    assert!(results.is_empty());
    assert_eq!(prefetcher.total_bytes(), 40);

    // Abort clears all pending and resets active bytes
    prefetcher.abort();
    assert_eq!(prefetcher.pending_count(), 0);
    assert_eq!(prefetcher.total_bytes(), 0);
    assert_eq!(prefetcher.completed_bytes(), 0);
}
