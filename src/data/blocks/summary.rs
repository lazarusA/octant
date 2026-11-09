//! Diagnostic summaries and memory aggregation for cached blocks.

use std::collections::HashMap;

use super::cache::BlockCache;
use super::key::BlockCacheKey;

/// Diagnostic summary of memory cached for a specific variable in a dataset source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableCacheSummary {
    pub source_id: String,
    pub variable_name: String,
    pub block_count: usize,
    pub bytes: usize,
}

impl BlockCache {
    /// Aggregated summary of cached data grouped by (source_id, variable_name),
    /// sorted in descending order of memory consumption.
    pub fn summary_per_variable(&self) -> Vec<VariableCacheSummary> {
        let mut map: HashMap<(String, String), (usize, usize)> = HashMap::new();
        for (key, block) in &self.entries {
            let entry = map
                .entry((key.source_id.clone(), key.variable_name.clone()))
                .or_insert((0, 0));
            entry.0 += 1;
            entry.1 += block.bytes_size();
        }

        let mut summaries: Vec<VariableCacheSummary> = map
            .into_iter()
            .map(
                |((source_id, variable_name), (block_count, bytes))| VariableCacheSummary {
                    source_id,
                    variable_name,
                    block_count,
                    bytes,
                },
            )
            .collect();

        summaries.sort_by(|a, b| {
            a.variable_name
                .cmp(&b.variable_name)
                .then_with(|| a.source_id.cmp(&b.source_id))
        });
        summaries
    }

    #[cfg(test)]
    /// Aggregated bytes by variable name.
    pub fn bytes_by_variable(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (key, block) in &self.entries {
            *map.entry(key.variable_name.clone()).or_insert(0) += block.bytes_size();
        }
        map
    }

    #[cfg(test)]
    /// Aggregated bytes by source ID.
    pub fn bytes_by_source(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (key, block) in &self.entries {
            *map.entry(key.source_id.clone()).or_insert(0) += block.bytes_size();
        }
        map
    }

    /// Removes and evicts all cached blocks for a specific variable.
    pub fn clear_variable(&mut self, source_id: &str, variable_name: &str) {
        let keys_to_remove: Vec<BlockCacheKey> = self
            .entries
            .keys()
            .filter(|k| k.source_id == source_id && k.variable_name == variable_name)
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(block) = self.entries.remove(&key) {
                self.current_bytes = self.current_bytes.saturating_sub(block.bytes_size());
            }
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }
    }
}
