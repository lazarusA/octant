//! Unit tests for block cache operations.

use crate::data::blocks::{BlockCache, BlockCacheKey};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use std::collections::HashMap as StdHashMap;

pub(super) fn test_block(origin0: usize) -> OctantBlock {
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
fn removing_a_source_drops_only_its_blocks() {
    let mut cache = BlockCache::new(1024 * 1024);
    cache.put(test_key("closed", "temperature", 0), test_block(0));
    cache.put(test_key("closed", "temperature", 2), test_block(2));
    let kept = test_key("open", "temperature", 0);
    cache.put(kept.clone(), test_block(0));
    let one_block = cache.current_bytes() / 3;
    cache.remove_source("closed");
    assert_eq!(cache.cached_count(), 1);
    assert!(cache.contains(&kept));
    assert_eq!(cache.current_bytes(), one_block);
}

#[test]
fn a_block_larger_than_the_cache_stays_resident() {
    // 96 bytes per block, more than the whole cache.
    let mut cache = BlockCache::new(64);
    let old = test_key("dataset-1", "temperature", 0);
    let big = test_key("dataset-1", "temperature", 2);
    cache.put(old.clone(), test_block(0));
    cache.put(big.clone(), test_block(2));
    assert!(!cache.contains(&old));
    assert!(cache.contains(&big));
    assert_eq!(cache.cached_count(), 1);
}
