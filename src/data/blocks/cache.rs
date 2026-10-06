//! Resident OctantBlock cache.

use std::collections::{HashMap, VecDeque};

pub use super::key::BlockCacheKey;
use super::key::selections_match_except_anim;
use crate::data::{octant_block::OctantBlock, slice_request::DimensionSelection};

/// LRU cache of resident N-dimensional OctantBlocks, budgeted by bytes.
///
/// `access_order.front()` is the least recently used block.
/// `access_order.back()` is the most recently used block.
pub struct BlockCache {
    pub(crate) entries: HashMap<BlockCacheKey, OctantBlock>,
    pub(crate) access_order: VecDeque<BlockCacheKey>,
    pub(crate) max_bytes: usize,
    pub(crate) current_bytes: usize,
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
    /// hyperslab bounds along `anim_dim` cover `timestep`, and whose selections on
    /// all other dimensions match `requested_selections`.
    pub fn find_covering_block(
        &mut self,
        source_id: &str,
        variable_name: &str,
        requested_selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> Option<OctantBlock> {
        let matching_key = self.entries.iter().find_map(|(key, block)| {
            if key.source_id == source_id && key.variable_name == variable_name {
                if !selections_match_except_anim(&key.selections, requested_selections, anim_dim) {
                    return None;
                }

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
    /// covers `timestep` along `anim_dim` and matches `requested_selections` on other dimensions,
    /// without mutating hit/miss statistics.
    pub fn covers(
        &self,
        source_id: &str,
        variable_name: &str,
        requested_selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        timestep: usize,
    ) -> bool {
        self.entries.iter().any(|(key, block)| {
            if key.source_id == source_id && key.variable_name == variable_name {
                if !selections_match_except_anim(&key.selections, requested_selections, anim_dim) {
                    return false;
                }

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

    /// Every resident block of `variable_name` whose selections match
    /// `requested_selections` on all dimensions but `anim_dim` (cheap clones:
    /// values are shared). No hit/miss accounting.
    pub fn matching_blocks(
        &self,
        source_id: &str,
        variable_name: &str,
        requested_selections: &[DimensionSelection],
        anim_dim: Option<usize>,
    ) -> Vec<OctantBlock> {
        self.entries
            .iter()
            .filter(|(key, _)| {
                key.source_id == source_id
                    && key.variable_name == variable_name
                    && selections_match_except_anim(&key.selections, requested_selections, anim_dim)
            })
            .map(|(_, block)| block.clone())
            .collect()
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

    /// Returns the minimum resident origin timestep for a variable along `anim_dim`, if any.
    pub fn min_resident_timestep(
        &self,
        source_id: &str,
        variable_name: &str,
        anim_dim: usize,
    ) -> Option<usize> {
        self.entries
            .iter()
            .filter(|(k, _)| k.source_id == source_id && k.variable_name == variable_name)
            .filter_map(|(_, block)| block.origin.get(anim_dim).copied())
            .min()
    }
}
