//! Parsed metadata documents behind a [`MetaReader`](super::MetaReader):
//! its own, or one a store shares between all its readers.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde_json::{Map, Value};

/// A group's attributes, `None` when it has none.
pub(super) type GroupAttrs = Option<Arc<Map<String, Value>>>;
/// An array's metadata document, `None` when it has none.
pub(super) type ArrayMeta = Option<Arc<Value>>;

/// Group attributes and array metadata by path, each document read and
/// parsed once, and the store's Zarr format once a group answered. A store
/// shares one between the readers of its array opens (`shared`), which run
/// on several threads; a lock is held to look up or insert, never while
/// reading.
#[derive(Debug)]
pub struct MetaCache {
    groups: Mutex<HashMap<String, GroupAttrs>>,
    arrays: Mutex<HashMap<String, ArrayMeta>>,
    /// `2` or `3`; `0` until a group answered.
    format: AtomicU8,
    /// Whether documents the store lacks are remembered.
    keeps_absent: bool,
}

impl MetaCache {
    fn new(keeps_absent: bool) -> Self {
        Self {
            groups: Mutex::default(),
            arrays: Mutex::default(),
            format: AtomicU8::new(0),
            keeps_absent,
        }
    }

    /// A reader's own cache.
    pub(super) fn own() -> Self {
        Self::new(true)
    }

    /// A cache shared by every reader of one store. A browser store fills in
    /// as keys are fetched, so there only documents found are remembered.
    pub fn shared() -> Self {
        Self::new(cfg!(not(target_arch = "wasm32")))
    }

    pub(super) fn group(&self, path: &str) -> Option<GroupAttrs> {
        lock(&self.groups).get(path).cloned()
    }

    pub(super) fn keep_group(&self, path: &str, attrs: &GroupAttrs) {
        self.keep(&self.groups, path, attrs);
    }

    pub(super) fn array(&self, path: &str) -> Option<ArrayMeta> {
        lock(&self.arrays).get(path).cloned()
    }

    pub(super) fn keep_array(&self, path: &str, meta: &ArrayMeta) {
        self.keep(&self.arrays, path, meta);
    }

    fn keep<T: Clone>(&self, map: &Mutex<HashMap<String, Option<T>>>, path: &str, v: &Option<T>) {
        if v.is_some() || self.keeps_absent {
            lock(map).insert(path.to_string(), v.clone());
        }
    }

    /// Records the attributes of group `path`, read in Zarr `format` by
    /// someone else (zarrs opening the root group), so no reader reads it.
    pub fn keep_found_group(&self, path: &str, attrs: Map<String, Value>, format: u8) {
        self.keep_group(path.trim_matches('/'), &Some(Arc::new(attrs)));
        self.set_format(format);
    }

    pub(super) fn format(&self) -> Option<u8> {
        match self.format.load(Ordering::Relaxed) {
            0 => None,
            format => Some(format),
        }
    }

    pub(super) fn set_format(&self, format: u8) {
        self.format.store(format, Ordering::Relaxed);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_browser_cache_forgets_missing_documents() {
        let attrs = Some(Arc::new(Map::new()));
        for (keeps_absent, missing_kept) in [(true, Some(None)), (false, None)] {
            let cache = MetaCache::new(keeps_absent);
            cache.keep_group("missing", &None);
            cache.keep_group("found", &attrs);
            assert_eq!(cache.group("missing"), missing_kept);
            assert_eq!(cache.group("found"), Some(attrs.clone()));
        }
    }
}
