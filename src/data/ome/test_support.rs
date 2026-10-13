//! In-memory OME-Zarr stores for the tests.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{Value, json};
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{ReadableStorageTraits, StoreKey, WritableStorageTraits};

use super::reader::{MetaReader, Prefetch};
use super::{Discovery, Outcome, discover};

/// Keys and contents of a store, written all at once by `memory`.
#[derive(Default, Clone)]
pub struct Fixture {
    pub keys: BTreeMap<String, Vec<u8>>,
}

impl Fixture {
    pub fn json(mut self, key: &str, value: Value) -> Self {
        self.keys
            .insert(key.to_string(), serde_json::to_vec(&value).expect("json"));
        self
    }

    pub fn bytes(mut self, key: &str, bytes: &[u8]) -> Self {
        self.keys.insert(key.to_string(), bytes.to_vec());
        self
    }

    /// A Zarr v2 group with attributes.
    pub fn v2_group(self, path: &str, attrs: Value) -> Self {
        let p = prefix(path);
        self.json(&format!("{p}.zgroup"), json!({ "zarr_format": 2 }))
            .json(&format!("{p}.zattrs"), attrs)
    }

    /// A Zarr v2 `uint16` array of `shape`, one chunk.
    pub fn v2_array(self, path: &str, shape: &[u64]) -> Self {
        self.v2_array_of(path, shape, "<u2")
    }

    /// A Zarr v2 array of `shape` and numpy `dtype`, one chunk.
    pub fn v2_array_of(self, path: &str, shape: &[u64], dtype: &str) -> Self {
        let p = prefix(path);
        self.json(
            &format!("{p}.zarray"),
            json!({
                "zarr_format": 2, "shape": shape, "chunks": shape, "dtype": dtype,
                "order": "C", "fill_value": 0, "filters": null, "compressor": null
            }),
        )
    }

    /// A Zarr v3 group with attributes.
    pub fn v3_group(self, path: &str, attrs: Value) -> Self {
        let p = prefix(path);
        self.json(
            &format!("{p}zarr.json"),
            json!({ "zarr_format": 3, "node_type": "group", "attributes": attrs }),
        )
    }

    /// A Zarr v3 `uint16` array of `shape`, one chunk.
    pub fn v3_array(self, path: &str, shape: &[u64], dims: &[&str]) -> Self {
        let p = prefix(path);
        self.json(
            &format!("{p}zarr.json"),
            json!({
                "zarr_format": 3, "node_type": "array", "shape": shape, "data_type": "uint16",
                "chunk_grid": { "name": "regular", "configuration": { "chunk_shape": shape } },
                "chunk_key_encoding": { "name": "default" }, "fill_value": 0,
                "codecs": [{ "name": "bytes", "configuration": { "endian": "little" } }],
                "dimension_names": dims
            }),
        )
    }

    pub fn memory(&self) -> Arc<MemoryStore> {
        let store = Arc::new(MemoryStore::new());
        for (k, v) in &self.keys {
            let key = StoreKey::new(k).expect("key");
            store.set(&key, v.clone().into()).expect("set");
        }
        store
    }

    /// Discovery over the store answering every key (desktop).
    pub fn discover(&self) -> Option<Discovery> {
        let storage: Arc<dyn ReadableStorageTraits> = self.memory();
        super::discover_direct(storage)
    }

    /// Discovery in web passes: each pass fetches the keys the previous one
    /// asked for. Returns the result and the number of passes.
    pub fn discover_prefetched(&self) -> (Option<Discovery>, usize) {
        let store = Arc::new(MemoryStore::new());
        let mut prefetch = Prefetch::default();
        for pass in 1..=32 {
            let storage: Arc<dyn ReadableStorageTraits> = store.clone();
            let outcome = discover(&MetaReader::prefetched(storage, &prefetch), "");
            let wanted = prefetch.take_pending();
            match outcome {
                Outcome::Found(found) => return (Some(found), pass),
                Outcome::NotOme => return (None, pass),
                Outcome::Pending => {}
            }
            assert!(!wanted.is_empty(), "pending without keys");
            for key in wanted {
                match self.keys.get(&key) {
                    Some(bytes) => store
                        .set(&StoreKey::new(&key).expect("key"), bytes.clone().into())
                        .expect("set"),
                    None => {
                        prefetch.absent.insert(key);
                    }
                }
            }
        }
        panic!("discovery never settled");
    }
}

fn prefix(path: &str) -> String {
    let path = path.trim_matches('/');
    if path.is_empty() {
        String::new()
    } else {
        format!("{path}/")
    }
}

/// 0.4 multiscale attributes with `levels` levels halving y and x.
pub fn multiscale_v04(axes: Value, level_scales: &[Vec<f64>]) -> Value {
    let datasets: Vec<Value> = level_scales
        .iter()
        .enumerate()
        .map(|(i, s)| json!({ "path": i.to_string(), "coordinateTransformations": [{ "type": "scale", "scale": s }] }))
        .collect();
    json!({ "version": "0.4", "axes": axes, "datasets": datasets })
}

pub fn czyx_axes() -> Value {
    json!([
        { "name": "c", "type": "channel" },
        { "name": "z", "type": "space", "unit": "micrometer" },
        { "name": "y", "type": "space", "unit": "micrometer" },
        { "name": "x", "type": "space", "unit": "micrometer" }
    ])
}
