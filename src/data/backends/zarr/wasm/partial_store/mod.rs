//! The browser's in-memory store, which also holds parts of objects: of a
//! shard, its index and the inner chunks fetched so far (`shard_fetch.rs`).
//! Range reads are answered from the parts held; whole objects behave as in
//! a `MemoryStore`. Shard parts and the data chunks stored with
//! [`PartialStore::insert_chunk`] count against the [`MemoryBudget`], which
//! drops the least recently used no [`Lease`] holds once past it; metadata
//! and other whole objects stay.

mod held;
mod lease;
mod traits;

use std::sync::{Arc, RwLock, RwLockWriteGuard};

use zarrs::storage::store::MemoryStore;
use zarrs::storage::{Bytes, ReadableStorageTraits, StorageError, StoreKey, WritableStorageTraits};

use crate::data::blocks::memory_budget::{Evictable, MemoryBudget};
use held::{Held, Use};
pub use lease::{Lease, array_prefixes};

/// A `MemoryStore` that also holds parts of objects.
pub struct PartialStore {
    whole: MemoryStore,
    held: RwLock<Held>,
    budget: Arc<MemoryBudget>,
}

impl PartialStore {
    /// A store counting against the budget every browser store shares.
    pub fn new() -> Arc<Self> {
        Self::with_budget(MemoryBudget::shared())
    }

    pub fn with_budget(budget: Arc<MemoryBudget>) -> Arc<Self> {
        let store = Arc::new(Self {
            whole: MemoryStore::new(),
            held: RwLock::default(),
            budget: budget.clone(),
        });
        budget.register(&store);
        store
    }

    fn held_mut(&self) -> RwLockWriteGuard<'_, Held> {
        self.held.write().unwrap_or_else(|p| p.into_inner())
    }

    /// Adds `bytes` at `offset` of the object `key` of `size` bytes, unless
    /// a piece already holds them (two loads fetched the same range).
    pub fn insert_part(&self, key: &StoreKey, size: u64, offset: u64, bytes: Bytes) {
        let len = bytes.len() as u64;
        {
            let tick = self.budget.tick();
            let mut guard = self.held_mut();
            let held = &mut *guard;
            let parts = held.parts.entry(key.clone()).or_default();
            parts.size = size;
            if parts.holds(offset, offset + len) {
                return;
            }
            parts.pieces.push((offset, bytes));
            let used = held.uses.entry(key.clone()).or_insert(Use {
                bytes: 0,
                last_used: tick,
            });
            used.bytes += len;
            used.last_used = tick;
        }
        self.budget.added(len);
        self.budget.trim();
    }

    /// Stores the whole data chunk `key`, which the budget may drop (and a
    /// load then fetches again).
    pub fn insert_chunk(&self, key: &StoreKey, bytes: Bytes) -> Result<(), StorageError> {
        let len = bytes.len() as u64;
        self.whole.set(key, bytes)?;
        let tick = self.budget.tick();
        let freed = {
            let mut held = self.held_mut();
            let freed = held.forget(key);
            held.uses.insert(
                key.clone(),
                Use {
                    bytes: len,
                    last_used: tick,
                },
            );
            freed
        };
        self.budget.released(freed);
        self.budget.added(len);
        self.budget.trim();
        Ok(())
    }

    /// Whether bytes `start..end` of the object `key` are held.
    pub fn holds(&self, key: &StoreKey, start: u64, end: u64) -> bool {
        if matches!(self.whole.size_key(key), Ok(Some(_))) {
            return true;
        }
        let held = self.held.read().unwrap_or_else(|p| p.into_inner());
        held.parts.get(key).is_some_and(|p| p.holds(start, end))
    }

    /// Keeps the objects `keys`, and those under `prefixes`, from the
    /// budget until the lease drops; they count as used then.
    pub fn lease(self: &Arc<Self>, keys: Vec<StoreKey>, prefixes: Vec<String>) -> Lease {
        let mut held = self.held_mut();
        for key in &keys {
            *held.pins.entry(key.clone()).or_default() += 1;
        }
        held.prefix_pins.extend(prefixes.iter().cloned());
        drop(held);
        Lease::new(self.clone(), keys, prefixes)
    }

    fn release(&self, keys: &[StoreKey], prefixes: &[String]) {
        let tick = self.budget.tick();
        {
            let mut guard = self.held_mut();
            let held = &mut *guard;
            for key in keys {
                if let Some(n) = held.pins.get_mut(key) {
                    *n -= 1;
                    if *n == 0 {
                        held.pins.remove(key);
                    }
                }
                if let Some(used) = held.uses.get_mut(key) {
                    used.last_used = tick;
                }
            }
            for prefix in prefixes {
                if let Some(i) = held.prefix_pins.iter().position(|p| p == prefix) {
                    held.prefix_pins.swap_remove(i);
                }
                for (key, used) in &mut held.uses {
                    if key.as_str().starts_with(prefix.as_str()) {
                        used.last_used = tick;
                    }
                }
            }
        }
        self.budget.trim();
    }
}

impl Evictable for PartialStore {
    fn evictable(&self, holder: usize, out: &mut Vec<(u64, usize, String)>) {
        let held = self.held.read().unwrap_or_else(|p| p.into_inner());
        out.extend(
            held.uses
                .iter()
                .filter(|(key, _)| !held.pinned(key))
                .map(|(key, used)| (used.last_used, holder, key.as_str().to_owned())),
        );
    }

    fn evict(&self, key: &str) -> u64 {
        let Ok(key) = StoreKey::new(key) else {
            return 0;
        };
        let mut held = self.held_mut();
        if held.pinned(&key) {
            return 0;
        }
        let freed = held.forget(&key);
        if freed > 0 {
            let _ = self.whole.erase(&key);
        }
        freed
    }
}

impl Drop for PartialStore {
    /// A released store's objects no longer count.
    fn drop(&mut self) {
        let held = self.held.get_mut().unwrap_or_else(|p| p.into_inner());
        self.budget
            .released(held.uses.values().map(|u| u.bytes).sum());
    }
}
