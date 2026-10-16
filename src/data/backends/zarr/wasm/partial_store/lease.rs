//! Leases: what a load reads, kept from the budget until it is decoded.

use std::sync::Arc;

use zarrs::storage::StoreKey;

use super::PartialStore;

/// Objects a load reads, kept by their store until the lease drops: its
/// block's chunks or shards by key, its coordinate arrays by prefix.
pub struct Lease {
    store: Arc<PartialStore>,
    keys: Vec<StoreKey>,
    prefixes: Vec<String>,
}

impl Lease {
    pub(super) fn new(
        store: Arc<PartialStore>,
        keys: Vec<StoreKey>,
        prefixes: Vec<String>,
    ) -> Self {
        Self {
            store,
            keys,
            prefixes,
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.store.release(&self.keys, &self.prefixes);
    }
}

/// The key prefixes of the arrays at `paths` (`(path, length)` pairs).
pub fn array_prefixes(paths: &[(String, u64)]) -> Vec<String> {
    paths
        .iter()
        .map(|(path, _)| format!("{}/", path.trim_matches('/')))
        .collect()
}
