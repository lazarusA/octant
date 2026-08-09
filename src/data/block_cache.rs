//! Resident OctantBlock cache.
//!
//! `BlockCacheKey` is built entirely from a request (source id + variable +
//! selections) — never from anything only known *after* a fetch, like
//! origin/shape, which backends clamp to the actual dimension extent during
//! I/O. Keying on the request instead of the result is what lets a caller check
//! `contains()`/`get()` *before* deciding whether to dispatch a fetch at all.

use std::collections::{HashMap, VecDeque};

use super::{
    octant_block::OctantBlock,
    slice_request::{DimensionSelection, SliceRequest},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockCacheKey {
    /// Identity of the opened source this block came from (`DataSource::id`,
    /// reached via `StoreHandle::source().id`). Two `StoreHandle`s opened
    /// against the same source resolve to the same key.
    pub source_id: String,
    pub variable_name: String,
    pub selections: Vec<DimensionSelection>,
}

impl BlockCacheKey {
    pub fn new(source_id: impl Into<String>, slice: &SliceRequest) -> Self {
        Self {
            source_id: source_id.into(),
            variable_name: slice.variable.clone(),
            selections: slice.selections.clone(),
        }
    }

    /// Reconstructs the SliceRequest represented by this key.
    pub fn to_slice_request(&self) -> SliceRequest {
        SliceRequest::new(self.variable_name.clone(), self.selections.clone())
    }
}

/// LRU cache of resident N-dimensional OctantBlocks, budgeted by bytes.
///
/// `access_order.front()` is the least recently used block.
/// `access_order.back()` is the most recently used block.
pub struct BlockCache {
    entries: HashMap<BlockCacheKey, OctantBlock>,
    access_order: VecDeque<BlockCacheKey>,
    max_bytes: usize,
    current_bytes: usize,
    hits: u64,
    misses: u64,
}

impl BlockCache {
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

    /// Checks presence without affecting hit/miss statistics.
    pub fn contains(&self, key: &BlockCacheKey) -> bool {
        self.entries.contains_key(key)
    }

    /// Finds any resident block in cache for `source_id` & `variable_name` whose
    /// hyperslab bounds along `anim_dim` cover `timestep`.
    pub fn find_covering_block(
        &mut self,
        source_id: &str,
        variable_name: &str,
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> Option<OctantBlock> {
        let matching_key = self.entries.iter().find_map(|(key, block)| {
            if key.source_id == source_id && key.variable_name == variable_name {
                if let Some(dim) = anim_dim {
                    let origin = block.origin.get(dim).copied().unwrap_or(0);
                    let extent = block.shape.get(dim).copied().unwrap_or(0);
                    if timestep >= origin && timestep < origin + extent {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    Some(key.clone())
                }
            } else {
                None
            }
        })?;

        self.get(&matching_key)
    }

    /// Checks whether any resident block in cache for `source_id` & `variable_name`
    /// covers `timestep` along `anim_dim`, without mutating hit/miss statistics.
    pub fn covers(
        &self,
        source_id: &str,
        variable_name: &str,
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> bool {
        self.entries.iter().any(|(key, block)| {
            if key.source_id == source_id && key.variable_name == variable_name {
                if let Some(dim) = anim_dim {
                    let origin = block.origin.get(dim).copied().unwrap_or(0);
                    let extent = block.shape.get(dim).copied().unwrap_or(0);
                    timestep >= origin && timestep < origin + extent
                } else {
                    true
                }
            } else {
                false
            }
        })
    }

    /// Gets a block and marks it as recently used. Counts as a real cache
    /// access (updates hits/misses).
    pub fn get(&mut self, key: &BlockCacheKey) -> Option<OctantBlock> {
        if let Some(block) = self.entries.get(key) {
            self.hits += 1;

            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());

            Some(block.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, key: BlockCacheKey, block: OctantBlock) {
        let bytes = block.bytes_size();

        if let Some(old) = self.entries.insert(key.clone(), block) {
            self.current_bytes = self.current_bytes.saturating_sub(old.bytes_size());

            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }

        self.current_bytes += bytes;
        self.access_order.push_back(key);

        self.evict();
    }

    fn evict(&mut self) {
        while self.current_bytes > self.max_bytes {
            let Some(key) = self.access_order.pop_front() else {
                break;
            };

            if let Some(block) = self.entries.remove(&key) {
                self.current_bytes = self.current_bytes.saturating_sub(block.bytes_size());
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_bytes = 0;
    }

    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn set_max_bytes(&mut self, bytes: usize) {
        self.max_bytes = bytes;
        self.evict();
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
            self.hits as f32 / total as f32 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn key_is_buildable_before_any_fetch() {
        let key = test_key("dataset-a", "temperature", 0);
        assert_eq!(key.source_id, "dataset-a");
        assert_eq!(key.variable_name, "temperature");
    }

    #[test]
    fn contains_does_not_affect_hit_rate() {
        let mut cache = BlockCache::new(1024 * 1024);
        let key = test_key("dataset-a", "temperature", 0);
        cache.put(key.clone(), test_block(0));

        assert!(cache.contains(&key));
        assert!(!cache.contains(&test_key("dataset-a", "temperature", 2)));

        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }
}
