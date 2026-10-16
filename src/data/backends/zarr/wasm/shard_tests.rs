//! Range-read shards in the browser store: the plan of what a window needs
//! from each shard, and blocks decoded from only those parts.

use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::byte_range::ByteRange;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{Bytes, ReadableStorageTraits, StoreKey, WritableStorageTraits};

use super::partial_store::PartialStore;
use super::shard_plan::{ShardIndexSpec, merge, parse_index, window_ranges};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

/// `v` (64 x 64 `float32`, `0, 1, 2, ...`) in 32 x 32 shards of 8 x 8
/// inner chunks, index at the end with a checksum (zarrs' default).
pub(super) fn source() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(vec![64, 64], vec![32, 32], f32_dt, FillValue::from(0f32));
    builder.subchunk_shape(vec![8, 8]);
    builder.dimension_names(Some(["row", "col"]));
    let array = builder.build(store.clone(), "/v").expect("build");
    array.store_metadata().expect("metadata");
    let values: Vec<f32> = (0..64 * 64).map(|i| i as f32).collect();
    let all = ArraySubset::new_with_shape(vec![64, 64]);
    array.store_array_subset(&all, &values).expect("values");
    store
}

fn key(k: &str) -> StoreKey {
    StoreKey::new(k).expect("key")
}

pub(super) fn whole(store: &MemoryStore, k: &str) -> Bytes {
    store.get(&key(k)).expect("get").expect("value")
}

pub(super) fn spec_of(store: &MemoryStore) -> ShardIndexSpec {
    let meta: serde_json::Value =
        serde_json::from_slice(&whole(store, "v/zarr.json")).expect("json");
    ShardIndexSpec::of(&meta).expect("plain index")
}

#[test]
fn the_index_spec_follows_the_sharding_codec() {
    let spec = spec_of(&source());
    assert_eq!(
        (spec.inner.clone(), spec.shard.clone()),
        (vec![8, 8], vec![32, 32])
    );
    assert!(spec.at_end && spec.checksum);
    assert_eq!(spec.grid(), [4, 4]);
    assert_eq!(spec.index_len(), 16 * 16 + 4);
    assert_eq!(spec.index_range(1000), 1000 - 260..1000);

    let plain = serde_json::json!({
        "chunk_grid": { "name": "regular", "configuration": { "chunk_shape": [8, 8] } },
        "codecs": [{ "name": "bytes", "configuration": { "endian": "little" } }]
    });
    assert_eq!(ShardIndexSpec::of(&plain), None, "not sharded");
}

#[test]
fn a_window_names_the_inner_chunks_it_touches() {
    let spec = spec_of(&source());
    // Shard (0, 1) holds columns 32..64; the window rows 10..20, columns 40..50.
    let chunks = spec.inner_chunks(&[0, 1], &[10..20, 40..50]);
    // Inner rows 1..3 (8..24) and columns 1..3 (40..56 in the shard's 8..24).
    assert_eq!(chunks, [4 + 1, 4 + 2, 8 + 1, 8 + 2]);
    assert!(spec.inner_chunks(&[1, 0], &[10..20, 40..50]).is_empty());
}

#[test]
fn close_ranges_merge_and_far_ones_stay_apart() {
    let merged = merge(vec![100..200, 0..50, 210..300, 10_000_000..10_000_010]);
    assert_eq!(merged, [0..300, 10_000_000..10_000_010]);
}

#[test]
fn partial_objects_answer_only_what_they_hold() {
    let store = PartialStore::new();
    let k = key("v/c/0/0");
    store.insert_part(&k, 1000, 900, Bytes::from(vec![7u8; 100]));
    assert_eq!(store.size_key(&k).expect("size"), Some(1000));
    let tail = store.get_partial(&k, ByteRange::Suffix(10)).expect("read");
    assert_eq!(tail.as_deref(), Some(&[7u8; 10][..]));
    assert!(
        store
            .get_partial(&k, ByteRange::FromStart(0, Some(4)))
            .is_err()
    );
    assert!(store.holds(&k, 950, 1000) && !store.holds(&k, 850, 950));
    store.set(&k, Bytes::from(vec![1u8; 4])).expect("set");
    assert_eq!(
        store.size_key(&k).expect("size"),
        Some(4),
        "the whole object replaces parts"
    );
}

#[test]
fn blocks_decode_from_the_index_and_the_window_chunks() {
    let src = source();
    let spec = spec_of(&src);
    let parts = Arc::new(PartialStore::new());
    for k in ["zarr.json", "v/zarr.json"] {
        if let Ok(Some(bytes)) = src.get(&key(k)) {
            parts.set(&key(k), bytes).expect("metadata");
        }
    }
    // What the browser fetches for rows 10..20, columns 40..50: shard (0, 1).
    let window = [10..20, 40..50];
    let shard = whole(&src, "v/c/0/1");
    let size = shard.len() as u64;
    let index_range = spec.index_range(size);
    let index = shard.slice(index_range.start as usize..index_range.end as usize);
    let shard_key = key("v/c/0/1");
    parts.insert_part(&shard_key, size, index_range.start, index.clone());
    let entries = parse_index(&spec, &index).expect("index");
    let ranges = window_ranges(&spec, &[0, 1], &window, &entries);
    // The index and 4 of the shard's 16 inner chunks (merging may read the
    // small gaps between them in one request).
    let needed: u64 = index.len() as u64 + ranges.iter().map(|r| r.end - r.start).sum::<u64>();
    assert!(needed * 2 < size, "{needed} of {size} bytes needed");
    for r in merge(ranges) {
        let piece = shard.slice(r.start as usize..r.end as usize);
        parts.insert_part(&shard_key, size, r.start, piece);
    }

    let store = GenericZarrBlockStore::new(parts, "memory://parts", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "v".into(),
        selections: vec![
            DimensionSelection::Range { start: 10, end: 20 },
            DimensionSelection::Range { start: 40, end: 50 },
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    let expected: Vec<f32> = (10..20)
        .flat_map(|r| (40..50).map(move |c| (r * 64 + c) as f32))
        .collect();
    assert_eq!(&block.values[..], &expected[..]);
}
