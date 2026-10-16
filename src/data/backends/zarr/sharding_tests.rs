//! Sharded arrays: listed with their inner chunks as the read unit, and read
//! by the inner chunks a window touches, never whole shards.

use std::sync::{Arc, Mutex};

use zarrs::array::{
    Array, ArrayBuilder, ArrayShardedReadableExtCache, ArraySubset, DataType, FillValue,
};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::byte_range::{ByteRange, ByteRangeIterator};
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{
    MaybeBytes, MaybeBytesIterator, ReadableStorageTraits, StorageError, StoreKey,
};

use super::ChunkReader;
use super::generic::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

/// A store recording every byte range read from it.
struct Recording {
    inner: Arc<MemoryStore>,
    reads: Mutex<Vec<(String, ByteRange)>>,
}

impl Recording {
    /// Bytes read from `key` so far, a whole value counted as `whole`.
    fn bytes_read(&self, key: &str, whole: u64) -> u64 {
        let reads = self.reads.lock().unwrap_or_else(|p| p.into_inner());
        reads
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, r)| r.length(whole))
            .sum()
    }
}

impl ReadableStorageTraits for Recording {
    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        let ranges: Vec<ByteRange> = byte_ranges.collect();
        let mut reads = self.reads.lock().unwrap_or_else(|p| p.into_inner());
        reads.extend(ranges.iter().map(|r| (key.as_str().to_string(), *r)));
        drop(reads);
        self.inner
            .get_partial_many(key, Box::new(ranges.into_iter()))
    }

    fn get(&self, key: &StoreKey) -> Result<MaybeBytes, StorageError> {
        self.get_partial(key, ByteRange::FromStart(0, None))
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        self.inner.size_key(key)
    }

    fn supports_get_partial(&self) -> bool {
        true
    }
}

/// `v` (64 x 64 `float32`, `0, 1, 2, ...`): one shard of 16 x 16 inner chunks.
fn sharded_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![64, 64], vec![64, 64], f32_dt, FillValue::from(0f32));
    builder.subchunk_shape(vec![16, 16]);
    builder.dimension_names(Some(["y", "x"]));
    let array = builder.build(store.clone(), "/v").expect("build");
    array.store_metadata().expect("metadata");
    let values: Vec<f32> = (0..64 * 64).map(|i| i as f32).collect();
    let all = ArraySubset::new_with_shape(vec![64, 64]);
    array.store_array_subset(&all, &values).expect("values");
    store
}

const SHARD: &str = "v/c/0/0";

fn shard_size(store: &MemoryStore) -> u64 {
    let key = StoreKey::new(SHARD).expect("key");
    store.size_key(&key).expect("size").expect("shard")
}

#[test]
fn sharded_arrays_list_their_inner_chunks() {
    let store = sharded_store();
    let array = Array::open(store.clone(), "/v").expect("open");
    let info = crate::utils::metadata::variable_info_from_array(&array, "v").expect("info");
    assert_eq!(info.chunk_shape, [16, 16], "the read unit");
    assert_eq!(info.shard_shape.as_deref(), Some(&[64, 64][..]));

    let key = StoreKey::new("v/zarr.json").expect("key");
    let bytes = store.get(&key).expect("get").expect("metadata");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    let shapes = crate::utils::metadata::sharding::sharding_shapes(&json);
    assert_eq!(shapes, Some((vec![16, 16], vec![64, 64])));
}

#[test]
fn a_window_reads_its_inner_chunks_not_the_shard() {
    let store = sharded_store();
    let whole = shard_size(&store);
    let recording = Arc::new(Recording {
        inner: store,
        reads: Mutex::default(),
    });
    let storage: Arc<dyn ReadableStorageTraits> = recording.clone();
    let array = Array::open(storage, "/v").expect("open");
    let cache = ArrayShardedReadableExtCache::new(&array);
    // One inner chunk: rows and columns 16..32.
    let subset = ArraySubset::new_with_ranges(&[16..32, 16..32]);
    let values = super::retrieve_array_subset_as_f32(&array, ChunkReader::Sharded(&cache), &subset)
        .expect("read");
    assert_eq!(values[0], (16 * 64 + 16) as f32);
    assert_eq!(values[255], (31 * 64 + 31) as f32);
    let sharded = recording.bytes_read(SHARD, whole);
    assert!(
        sharded * 4 < whole,
        "{sharded} of {whole} bytes: index and one chunk"
    );

    // The decoded-chunk cache other arrays read through decodes whole
    // chunks: here the shard.
    recording
        .reads
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clear();
    let array = Arc::new(array);
    let decoded =
        zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit::new(array.clone(), 1 << 20);
    super::retrieve_array_subset_as_f32(&array, ChunkReader::Cached(&decoded), &subset)
        .expect("read");
    let cached = recording.bytes_read(SHARD, whole);
    assert_eq!(cached, whole, "a cached read takes the shard");
}

#[test]
fn sharded_blocks_read_back_exactly() {
    let store = GenericZarrBlockStore::new(sharded_store(), "memory://sharded", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "v".into(),
        selections: vec![
            DimensionSelection::Range { start: 10, end: 40 },
            DimensionSelection::Index(33),
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    let mut expected: Vec<f32> = (10..40).map(|y| (y * 64 + 33) as f32).collect();
    // Blocks run north-up: `y` may come back flipped.
    if block.flipped_dims.iter().any(|d| d == "y") {
        expected.reverse();
    }
    assert_eq!(&block.values[..], &expected[..]);
}
