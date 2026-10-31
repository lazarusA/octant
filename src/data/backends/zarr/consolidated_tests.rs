//! A store's consolidated metadata spares its readers the documents it
//! lists: inspection reads no group, the coordinate search probes no key,
//! and a failed read of the root is asked again.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::storage::byte_range::ByteRangeIterator;
use zarrs::storage::storage_adapter::usage_log::UsageLogStorageAdapter;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{
    ListableStorageTraits, MaybeBytes, MaybeBytesIterator, ReadableStorageTraits, StorageError,
    StoreKey, StoreKeys, StoreKeysPrefixes, StorePrefix, WritableStorageTraits,
};

use super::generic::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

/// A float array at `path` of `shape` with dimensions `dims`, holding `values`.
fn array(store: &Arc<MemoryStore>, path: &str, shape: &[u64], dims: &[&str], values: &[f32]) {
    let dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(shape.to_vec(), shape.to_vec(), dt, FillValue::from(0f32));
    builder.dimension_names(Some(dims.to_vec()));
    let a = builder.build(store.clone(), path).expect("build");
    a.store_metadata().expect("metadata");
    let subset = ArraySubset::new_with_shape(shape.to_vec());
    a.store_array_subset(&subset, values).expect("values");
}

/// Groups `g`, `g/h` and `other` with attributes, array `g/data` (`lat`,
/// `x`), its latitude `g/lat`, another `lat` at the root and an `x` only in
/// `other`, consolidated at the root.
fn consolidated_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    for (path, source) in [
        ("/", "root"),
        ("/g", "g"),
        ("/g/h", "h"),
        ("/other", "other"),
    ] {
        let mut attrs = serde_json::Map::new();
        attrs.insert("source".into(), source.into());
        let mut builder = zarrs::group::GroupBuilder::new();
        builder.attributes(attrs);
        let group = builder.build(store.clone(), path).expect("group");
        group.store_metadata().expect("group metadata");
    }
    array(&store, "/g/data", &[4, 4], &["lat", "x"], &[0.0; 16]);
    array(&store, "/g/lat", &[4], &["lat"], &[1.0, 2.0, 3.0, 4.0]);
    array(&store, "/lat", &[4], &["lat"], &[10.0, 20.0, 30.0, 40.0]);
    array(&store, "/other/x", &[4], &["x"], &[5.0, 6.0, 7.0, 8.0]);
    let node = zarrs::node::Node::open(&store, "/").expect("node");
    let metadata = node.consolidate_metadata().expect("consolidated");
    let mut root = zarrs::group::Group::open(store.clone(), "/").expect("root");
    root.set_consolidated_metadata(Some(ConsolidatedMetadata {
        metadata,
        ..ConsolidatedMetadata::default()
    }));
    root.store_metadata().expect("root");
    store
}

/// `memory` behind a log of its reads, as a block store.
fn logged(memory: Arc<MemoryStore>) -> (GenericZarrBlockStore, Arc<Mutex<Vec<u8>>>) {
    let log = Arc::new(Mutex::new(Vec::<u8>::new()));
    let logged = UsageLogStorageAdapter::new(memory, log.clone(), String::new);
    let store = GenericZarrBlockStore::new(Arc::new(logged), "memory://cons", "zarr", "Zarr");
    (store, log)
}

/// The metadata documents read so far, by key.
fn documents_read(log: &Mutex<Vec<u8>>) -> Vec<String> {
    let log = log.lock().unwrap_or_else(|p| p.into_inner());
    let text = String::from_utf8_lossy(&log);
    (text.lines())
        .filter_map(|l| l.split_once('(')?.1.split([',', ')']).next())
        .filter(|key| key.ends_with("zarr.json"))
        .map(str::to_string)
        .collect()
}

#[test]
fn inspection_reads_only_the_consolidated_root() {
    let (store, log) = logged(consolidated_store());
    store.inspect().expect("inspect");
    assert_eq!(documents_read(&log), ["zarr.json"]);
}

#[test]
fn coordinates_are_found_in_the_consolidated_metadata() {
    let (store, log) = logged(consolidated_store());
    let request = SliceRequest {
        variable: "g/data".into(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 4 },
            DimensionSelection::Range { start: 0, end: 4 },
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    // The variable's own group's latitude, not the root's; `x` has none
    // (`other/x` is a sibling group's), and no key was asked for to learn that.
    let lat = block.coordinates.get("lat").expect("lat");
    assert!(lat.contains(&1.0) && lat.contains(&4.0), "{lat:?}");
    assert!(
        !block.coordinates.contains_key("x"),
        "{:?}",
        block.coordinates.keys()
    );
    assert_eq!(documents_read(&log), ["zarr.json"]);
}

#[test]
fn a_root_known_unconsolidated_is_not_read_for_coordinates() {
    use crate::data::backends::coord_bounds::discover::discover_coord_array;
    let memory = Arc::new(MemoryStore::new());
    for path in ["/", "/g"] {
        let group = zarrs::group::GroupBuilder::new()
            .build(memory.clone(), path)
            .expect("group");
        group.store_metadata().expect("group metadata");
    }
    let root_reads = |known| {
        let log = Arc::new(Mutex::new(Vec::<u8>::new()));
        let logged = UsageLogStorageAdapter::new(memory.clone(), log.clone(), String::new);
        assert!(discover_coord_array(Arc::new(logged), known, &[], true).is_none());
        (documents_read(&log).iter())
            .filter(|k| *k == "zarr.json")
            .count()
    };
    // Unread, the root is read for its consolidated metadata; known to have
    // none, it is not.
    assert_eq!(root_reads(None), 1);
    assert_eq!(root_reads(Some(None)), 0);
}

#[test]
fn a_missing_coordinate_reads_each_key_once() {
    use crate::data::backends::coord_bounds::discover::discover_coord_array;
    let memory = Arc::new(MemoryStore::new());
    for path in ["/", "/g"] {
        let group = zarrs::group::GroupBuilder::new()
            .build(memory.clone(), path)
            .expect("group");
        group.store_metadata().expect("group metadata");
    }
    let log = Arc::new(Mutex::new(Vec::<u8>::new()));
    let logged = UsageLogStorageAdapter::new(memory, log.clone(), String::new);
    let candidates = ["g/row".to_string(), "row".to_string()];
    assert!(discover_coord_array(Arc::new(logged), Some(None), &candidates, false).is_none());
    let log = log.lock().unwrap_or_else(|p| p.into_inner());
    let text = String::from_utf8_lossy(&log);
    let mut keys: Vec<&str> = (text.lines())
        .filter_map(|l| l.split_once("get(")?.1.split(')').next())
        .collect();
    assert!(!keys.contains(&"zarr.json"), "the root was read: {keys:?}");
    let read = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), read, "a key was read twice");
}

#[test]
fn a_root_array_is_no_coordinate() {
    use crate::data::backends::coord_bounds::discover::discover_coord_array;
    let memory = Arc::new(MemoryStore::new());
    array(&memory, "/", &[2, 2], &["row", "col"], &[0.0; 4]);
    let candidates = ["row".to_string()];
    assert!(discover_coord_array(memory, Some(None), &candidates, true).is_none());
}

#[test]
fn root_variables_find_coordinates_in_a_child_group() {
    let store = Arc::new(MemoryStore::new());
    for path in ["/", "/coords"] {
        let group = zarrs::group::GroupBuilder::new()
            .build(store.clone(), path)
            .expect("group");
        group.store_metadata().expect("group metadata");
    }
    array(&store, "/t2m", &[3], &["time"], &[0.0; 3]);
    array(&store, "/coords/time", &[3], &["time"], &[5.0, 6.0, 7.0]);
    let node = zarrs::node::Node::open(&store, "/").expect("node");
    let metadata = node.consolidate_metadata().expect("consolidated");
    let time = crate::data::backends::coord_bounds::read_coord_values_scoped(
        store,
        "time",
        None,
        &[],
        Some(Some(&metadata)),
    )
    .expect("time");
    assert_eq!(
        (time.first_number(), time.last_number()),
        (Some(5.0), Some(7.0))
    );
}

/// A store whose root `zarr.json` cannot be read while `failing`, counting
/// the reads of it.
struct FailingRoot {
    inner: Arc<MemoryStore>,
    failing: AtomicBool,
    root_reads: AtomicUsize,
}

impl FailingRoot {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: consolidated_store(),
            failing: AtomicBool::new(true),
            root_reads: AtomicUsize::new(0),
        })
    }

    fn check(&self, key: &StoreKey) -> Result<(), StorageError> {
        if key.as_str() == "zarr.json" {
            self.root_reads.fetch_add(1, Ordering::SeqCst);
        }
        match self.failing.load(Ordering::SeqCst) && key.as_str() == "zarr.json" {
            true => Err(StorageError::Other("timed out".into())),
            false => Ok(()),
        }
    }
}

impl ReadableStorageTraits for FailingRoot {
    fn get(&self, key: &StoreKey) -> Result<MaybeBytes, StorageError> {
        self.check(key)?;
        self.inner.get(key)
    }

    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        self.check(key)?;
        self.inner.get_partial_many(key, byte_ranges)
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        self.inner.size_key(key)
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

impl ListableStorageTraits for FailingRoot {
    fn list(&self) -> Result<StoreKeys, StorageError> {
        self.inner.list()
    }

    fn list_prefix(&self, prefix: &StorePrefix) -> Result<StoreKeys, StorageError> {
        self.inner.list_prefix(prefix)
    }

    fn list_dir(&self, prefix: &StorePrefix) -> Result<StoreKeysPrefixes, StorageError> {
        self.inner.list_dir(prefix)
    }

    fn size_prefix(&self, prefix: &StorePrefix) -> Result<u64, StorageError> {
        self.inner.size_prefix(prefix)
    }
}

impl WritableStorageTraits for FailingRoot {
    fn set(&self, key: &StoreKey, value: zarrs::storage::Bytes) -> Result<(), StorageError> {
        self.inner.set(key, value)
    }

    fn set_partial_many(
        &self,
        key: &StoreKey,
        offset_values: zarrs::storage::OffsetBytesIterator,
    ) -> Result<(), StorageError> {
        self.inner.set_partial_many(key, offset_values)
    }

    fn erase(&self, key: &StoreKey) -> Result<(), StorageError> {
        self.inner.erase(key)
    }

    fn erase_prefix(&self, prefix: &StorePrefix) -> Result<(), StorageError> {
        self.inner.erase_prefix(prefix)
    }

    fn supports_set_partial(&self) -> bool {
        false
    }
}

#[test]
fn a_failed_root_read_is_asked_again() {
    let storage = FailingRoot::new();
    let store = GenericZarrBlockStore::new(storage.clone(), "memory://flaky", "zarr", "Zarr");
    assert!(store.consolidated().is_none(), "the root could not be read");
    storage.failing.store(false, Ordering::SeqCst);
    assert!(store.consolidated().is_some(), "read again, not remembered");
}

#[test]
fn blocks_of_an_open_array_do_not_retry_a_failing_root() {
    let storage = FailingRoot::new();
    let store = GenericZarrBlockStore::new(storage.clone(), "memory://down", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "g/data".into(),
        selections: vec![DimensionSelection::Index(0), DimensionSelection::Index(0)],
    };
    store.fetch_block(&request).expect("first block");
    let after_open = storage.root_reads.load(Ordering::SeqCst);
    store.fetch_block(&request).expect("second block");
    // The array's open tried the root; its blocks ask it nothing more.
    assert_eq!(storage.root_reads.load(Ordering::SeqCst), after_open);
}
