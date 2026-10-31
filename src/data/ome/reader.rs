//! Metadata reads for OME-Zarr discovery, shared by the desktop and the web.
//!
//! On the desktop the store answers every key at once. In the browser keys
//! are fetched asynchronously into a memory store first: discovery runs over
//! what is already there, notes the keys it still needs (`Lookup::Pending`),
//! and is run again once they have been fetched (`zarr/wasm/ome.rs`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Map, Value};
use zarrs::storage::{ReadableStorageTraits, StoreKey};

pub(super) use super::lookup::Lookup;
pub use super::lookup::Prefetch;
use super::meta_cache::{ArrayMeta, GroupAttrs, MetaCache};

/// Reads metadata keys from a store, optionally behind a web prefetch. Each
/// key is read and parsed once per reader (the web keeps one across its
/// walks; a store's array opens share one `MetaCache`), and once a group
/// answers in one Zarr format only that format's keys are asked: a remote
/// store is spared requests that would fail.
pub struct MetaReader<'a> {
    storage: Arc<dyn ReadableStorageTraits>,
    prefetch: Option<&'a Prefetch>,
    seen: RefCell<HashMap<String, Lookup>>,
    /// Parsed documents and the store's format, unless `shared` holds them.
    own: MetaCache,
    shared: Option<&'a MetaCache>,
}

impl<'a> MetaReader<'a> {
    /// A reader answering every key from `storage` (desktop).
    pub fn direct(storage: Arc<dyn ReadableStorageTraits>) -> Self {
        Self::new(storage, None)
    }

    /// This reader answering through `cache`, shared by every reader of the
    /// store (its inspection and array opens), so a document one of them
    /// parsed is not read again.
    pub fn with_cache(self, cache: &'a MetaCache) -> Self {
        Self {
            shared: Some(cache),
            ..self
        }
    }

    /// A reader over keys prefetched into `storage` (web).
    pub fn prefetched(storage: Arc<dyn ReadableStorageTraits>, prefetch: &'a Prefetch) -> Self {
        Self::new(storage, Some(prefetch))
    }

    fn new(storage: Arc<dyn ReadableStorageTraits>, prefetch: Option<&'a Prefetch>) -> Self {
        Self {
            storage,
            prefetch,
            seen: RefCell::default(),
            own: MetaCache::own(),
            shared: None,
        }
    }

    fn cache(&self) -> &MetaCache {
        self.shared.unwrap_or(&self.own)
    }

    /// Asks only Zarr `format`'s keys (`2` or `3`), as once a group answered.
    pub fn assume_format(&self, format: u8) {
        self.cache().set_format(format);
    }

    /// The storage arrays are opened from.
    pub fn storage(&self) -> Arc<dyn ReadableStorageTraits> {
        self.storage.clone()
    }

    pub(super) fn get(&self, key: &str) -> Lookup {
        let key = key.trim_start_matches('/');
        if let Some(known) = self.seen.borrow().get(key) {
            return known.clone();
        }
        let answer = self.read(key);
        if answer != Lookup::Pending {
            self.seen
                .borrow_mut()
                .insert(key.to_string(), answer.clone());
        }
        answer
    }

    fn read(&self, key: &str) -> Lookup {
        let Ok(store_key) = StoreKey::new(key) else {
            return Lookup::Absent;
        };
        let read = self.storage.get(&store_key);
        match (read, self.prefetch) {
            (Ok(Some(bytes)), _) => Lookup::Found(bytes),
            (Ok(None), None) => Lookup::Absent,
            (Err(_), None) => Lookup::Failed,
            (_, Some(prefetch)) => prefetch.answer(key),
        }
    }

    /// The first of `keys` the store has. Once one is pending, the rest are
    /// asked too, so the web fetches them together rather than one per pass;
    /// one that failed makes the answer `Failed` rather than `Absent`.
    fn first_of(&self, keys: &[String]) -> Lookup {
        let (mut pending, mut failed) = (false, false);
        for key in keys {
            match self.get(key) {
                Lookup::Found(bytes) if !pending => return Lookup::Found(bytes),
                Lookup::Pending => pending = true,
                Lookup::Failed => failed = true,
                _ => {}
            }
        }
        match (pending, failed) {
            (true, _) => Lookup::Pending,
            (false, true) => Lookup::Failed,
            (false, false) => Lookup::Absent,
        }
    }

    /// Whether the store has a group (or array) metadata document at `path`,
    /// without parsing it.
    pub fn has_group(&self, path: &str) -> Result<bool, Pending> {
        let keys = self.keys(join(path, "zarr.json"), join(path, ".zattrs"));
        match self.first_of(&keys) {
            Lookup::Found(_) => Ok(true),
            Lookup::Absent | Lookup::Failed => Ok(false),
            Lookup::Pending => Err(Pending),
        }
    }

    /// A group's attributes: `zarr.json` `attributes` (Zarr v3) or `.zattrs`
    /// (Zarr v2). `Ok(None)` when the group has neither.
    pub fn group_attributes(&self, path: &str) -> Result<GroupAttrs, Pending> {
        let path = path.trim_matches('/');
        if let Some(known) = self.cache().group(path) {
            return Ok(known);
        }
        let keys = self.keys(join(path, "zarr.json"), join(path, ".zattrs"));
        let attrs = match self.first_of(&keys) {
            Lookup::Found(bytes) => attributes_of(&bytes).map(|(attrs, format)| {
                self.cache().set_format(format);
                Arc::new(attrs)
            }),
            Lookup::Absent => None,
            // Not remembered: the next reader asks again.
            Lookup::Failed => return Ok(None),
            Lookup::Pending => return Err(Pending),
        };
        self.cache().keep_group(path, &attrs);
        Ok(attrs)
    }

    /// An array's metadata document (`zarr.json` or `.zarray`).
    pub fn array_metadata(&self, path: &str) -> Result<ArrayMeta, Pending> {
        let path = path.trim_matches('/');
        if let Some(known) = self.cache().array(path) {
            return Ok(known);
        }
        let keys = self.keys(join(path, "zarr.json"), join(path, ".zarray"));
        let meta = match self.first_of(&keys) {
            Lookup::Found(bytes) => serde_json::from_slice::<Value>(&bytes).ok().map(Arc::new),
            Lookup::Absent => None,
            Lookup::Failed => return Ok(None),
            Lookup::Pending => return Err(Pending),
        };
        self.cache().keep_array(path, &meta);
        Ok(meta)
    }

    /// The shape an array's metadata declares.
    pub fn array_shape(&self, path: &str) -> Result<Option<Vec<u64>>, Pending> {
        Ok(self.array_metadata(path)?.and_then(|meta| {
            let shape = meta.get("shape")?.as_array()?;
            Some(shape.iter().filter_map(Value::as_u64).collect())
        }))
    }

    /// The keys to try: the store's format's alone once known, else both.
    fn keys(&self, v3: String, v2: String) -> Vec<String> {
        match self.cache().format() {
            Some(2) => vec![v2],
            Some(_) => vec![v3],
            None => vec![v3, v2],
        }
    }
}

/// The attributes in a group's metadata document and its Zarr format:
/// `zarr.json` `attributes` (`3`) or the whole `.zattrs` (`2`). `None` when
/// the document is not a JSON object.
pub fn attributes_of(bytes: &[u8]) -> Option<(Map<String, Value>, u8)> {
    let Ok(Value::Object(mut doc)) = serde_json::from_slice::<Value>(bytes) else {
        return None;
    };
    if !doc.contains_key("zarr_format") && !doc.contains_key("node_type") {
        return Some((doc, 2));
    }
    let attrs = match doc.remove("attributes") {
        Some(Value::Object(a)) => a,
        _ => Map::new(),
    };
    Some((attrs, 3))
}

/// A read that must wait for keys being fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending;

/// `path/name` without leading or doubled slashes.
pub fn join(path: &str, name: &str) -> String {
    let path = path.trim_matches('/');
    let name = name.trim_matches('/');
    match (path.is_empty(), name.is_empty()) {
        (true, _) => name.to_string(),
        (false, true) => path.to_string(),
        (false, false) => format!("{path}/{name}"),
    }
}
