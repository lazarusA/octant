//! Metadata reads for OME-Zarr discovery, shared by the desktop and the web.
//!
//! On the desktop the store answers every key at once. In the browser keys
//! are fetched asynchronously into a memory store first: discovery runs over
//! what is already there, notes the keys it still needs (`Lookup::Pending`),
//! and is run again once they have been fetched (`zarr/wasm/ome.rs`).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use serde_json::{Map, Value};
use zarrs::storage::{ReadableStorageTraits, StoreKey};

/// The answer for one key.
#[derive(Debug, Clone, PartialEq)]
pub enum Lookup {
    Found(Vec<u8>),
    /// The store has no such key.
    Absent,
    /// Not fetched yet (web only).
    Pending,
}

/// Keys already fetched without success, and keys still needed (web only).
#[derive(Debug, Default)]
pub struct Prefetch {
    absent: RefCell<HashSet<String>>,
    pending: RefCell<BTreeSet<String>>,
}

impl Prefetch {
    /// The keys needed since the last call, emptying the list.
    pub fn take_pending(&self) -> Vec<String> {
        std::mem::take(&mut *self.pending.borrow_mut())
            .into_iter()
            .collect()
    }

    /// Records that the store has no `key` (or it could not be fetched).
    pub fn mark_absent(&self, key: String) {
        self.absent.borrow_mut().insert(key);
    }
}

/// Reads metadata keys from a store, optionally behind a web prefetch. Each
/// key is read once per reader (the web keeps one across its walks), and once
/// a group answers in one Zarr format only that format's keys are asked: a
/// remote store is spared requests that would fail.
pub struct MetaReader<'a> {
    storage: Arc<dyn ReadableStorageTraits>,
    prefetch: Option<&'a Prefetch>,
    seen: RefCell<HashMap<String, Lookup>>,
    /// The store's Zarr format, once a group answered (`2` or `3`).
    format: Cell<Option<u8>>,
}

impl<'a> MetaReader<'a> {
    /// A reader answering every key from `storage` (desktop).
    pub fn direct(storage: Arc<dyn ReadableStorageTraits>) -> Self {
        Self {
            storage,
            prefetch: None,
            seen: RefCell::default(),
            format: Cell::new(None),
        }
    }

    /// A reader over keys prefetched into `storage` (web).
    pub fn prefetched(storage: Arc<dyn ReadableStorageTraits>, prefetch: &'a Prefetch) -> Self {
        Self {
            storage,
            prefetch: Some(prefetch),
            seen: RefCell::default(),
            format: Cell::new(None),
        }
    }

    /// The storage arrays are opened from.
    pub fn storage(&self) -> Arc<dyn ReadableStorageTraits> {
        self.storage.clone()
    }

    pub fn get(&self, key: &str) -> Lookup {
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
        if let Ok(store_key) = StoreKey::new(key)
            && let Ok(Some(bytes)) = self.storage.get(&store_key)
        {
            return Lookup::Found(bytes.to_vec());
        }
        let Some(prefetch) = self.prefetch else {
            return Lookup::Absent;
        };
        if prefetch.absent.borrow().contains(key) {
            return Lookup::Absent;
        }
        prefetch.pending.borrow_mut().insert(key.to_string());
        Lookup::Pending
    }

    /// The first of `keys` the store has. Once one is pending, the rest are
    /// asked too, so the web fetches them together rather than one per pass.
    pub fn first_of(&self, keys: &[String]) -> Lookup {
        let mut pending = false;
        for key in keys {
            match self.get(key) {
                Lookup::Found(bytes) if !pending => return Lookup::Found(bytes),
                Lookup::Pending => pending = true,
                _ => {}
            }
        }
        if pending {
            Lookup::Pending
        } else {
            Lookup::Absent
        }
    }

    /// A group's attributes: `zarr.json` `attributes` (Zarr v3) or `.zattrs`
    /// (Zarr v2). `Ok(None)` when the group has neither.
    pub fn group_attributes(&self, path: &str) -> Result<Option<Map<String, Value>>, Pending> {
        let keys = self.keys(join(path, "zarr.json"), join(path, ".zattrs"));
        match self.first_of(&keys) {
            Lookup::Found(bytes) => Ok(attributes_of(&bytes).map(|(attrs, format)| {
                self.format.set(Some(format));
                attrs
            })),
            Lookup::Absent => Ok(None),
            Lookup::Pending => Err(Pending),
        }
    }

    /// An array's metadata document (`zarr.json` or `.zarray`).
    pub fn array_metadata(&self, path: &str) -> Result<Option<Value>, Pending> {
        let keys = self.keys(join(path, "zarr.json"), join(path, ".zarray"));
        match self.first_of(&keys) {
            Lookup::Found(bytes) => Ok(serde_json::from_slice::<Value>(&bytes).ok()),
            Lookup::Absent => Ok(None),
            Lookup::Pending => Err(Pending),
        }
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
        match self.format.get() {
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
