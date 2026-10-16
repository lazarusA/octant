//! Decoded Icechunk manifests by object id, counted against the
//! [`MemoryBudget`]. Only manifests no load holds (through their `Arc`) are
//! dropped, and a later load fetches a dropped one again.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use icechunk_format::manifest::Manifest;

use crate::data::blocks::memory_budget::{Evictable, MemoryBudget};

struct Cached {
    manifest: Arc<Manifest>,
    /// Its decompressed size.
    bytes: u64,
    last_used: AtomicU64,
}

pub struct ManifestCache {
    entries: RwLock<HashMap<String, Cached>>,
    budget: Arc<MemoryBudget>,
}

impl ManifestCache {
    /// A cache counting against the budget every holder shares.
    pub fn new() -> Arc<Self> {
        Self::with_budget(MemoryBudget::shared())
    }

    pub fn with_budget(budget: Arc<MemoryBudget>) -> Arc<Self> {
        let cache = Arc::new(Self {
            entries: RwLock::default(),
            budget: budget.clone(),
        });
        budget.register(&cache);
        cache
    }

    /// The manifest `id`, if held, marked as used.
    pub fn get(&self, id: &str) -> Option<Arc<Manifest>> {
        let entries = self.entries.read().unwrap_or_else(|p| p.into_inner());
        let cached = entries.get(id)?;
        cached
            .last_used
            .store(self.budget.tick(), Ordering::Relaxed);
        Some(cached.manifest.clone())
    }

    /// Holds `manifest` as `id`, decoded from `bytes` bytes.
    pub fn insert(&self, id: String, manifest: Arc<Manifest>, bytes: u64) {
        let cached = Cached {
            manifest,
            bytes,
            last_used: AtomicU64::new(self.budget.tick()),
        };
        let replaced = self
            .entries
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(id, cached);
        self.budget.released(replaced.map_or(0, |c| c.bytes));
        self.budget.added(bytes);
        self.budget.trim();
    }
}

impl Evictable for ManifestCache {
    fn evictable(&self, holder: usize, out: &mut Vec<(u64, usize, String)>) {
        let entries = self.entries.read().unwrap_or_else(|p| p.into_inner());
        // A manifest a load still holds would stay in memory uncounted.
        out.extend(
            entries
                .iter()
                .filter(|(_, c)| Arc::strong_count(&c.manifest) == 1)
                .map(|(id, c)| (c.last_used.load(Ordering::Relaxed), holder, id.clone())),
        );
    }

    fn evict(&self, key: &str) -> u64 {
        let mut entries = self.entries.write().unwrap_or_else(|p| p.into_inner());
        if entries
            .get(key)
            .is_none_or(|c| Arc::strong_count(&c.manifest) > 1)
        {
            return 0;
        }
        entries.remove(key).map_or(0, |c| c.bytes)
    }
}

impl Drop for ManifestCache {
    /// A released cache's manifests no longer count.
    fn drop(&mut self) {
        let entries = self.entries.get_mut().unwrap_or_else(|p| p.into_inner());
        self.budget
            .released(entries.values().map(|c| c.bytes).sum());
    }
}
