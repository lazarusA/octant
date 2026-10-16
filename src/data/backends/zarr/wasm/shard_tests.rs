//! Range-read shards in the browser store: the plan of what a window needs
//! from each shard, and blocks decoded from only those parts.

use std::sync::Arc;

use zarrs::array::codec::{ShardingCodecBuilder, ShardingIndexLocation};
use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::byte_range::ByteRange;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{Bytes, ReadableStorageTraits, StoreKey, WritableStorageTraits};

use super::partial_store::PartialStore;
use super::shard_plan::{ShardIndexSpec, chunk_entry, merge, window_ranges};

/// `v` (64 x 64 `float32`, `0, 1, 2, ...`) in 32 x 32 shards of 8 x 8
/// inner chunks, index at the end with a checksum (zarrs' default).
pub(super) fn source() -> Arc<MemoryStore> {
    source_at(ShardingIndexLocation::End)
}

/// [`source`] with its shard indexes at `location`.
pub(super) fn source_at(location: ShardingIndexLocation) -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let eight = std::num::NonZeroU64::new(8).expect("8");
    let inner = vec![eight, eight];
    let sharding = ShardingCodecBuilder::new(inner, &f32_dt)
        .index_location(location)
        .build_arc();
    let mut builder = ArrayBuilder::new(vec![64, 64], vec![32, 32], f32_dt, FillValue::from(0f32));
    builder.array_to_bytes_codec(sharding);
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
fn entries_past_the_shard_are_dropped() {
    let spec = spec_of(&source());
    // Every entry empty, then the window's chunks 5, 6 and 9 set.
    let mut index = vec![0xFF; 16 * 16];
    let mut set = |i: usize, offset: u64, len: u64| {
        index[16 * i..16 * i + 8].copy_from_slice(&offset.to_le_bytes());
        index[16 * i + 8..16 * i + 16].copy_from_slice(&len.to_le_bytes());
    };
    set(5, 100, 50);
    set(6, u64::MAX - 1, 10); // overflows
    set(9, 990, 20); // ends past a 1000-byte shard
    assert_eq!(chunk_entry(&index, 5), Some((100, 50)));
    assert_eq!(chunk_entry(&index, 10), None, "empty");
    assert_eq!(chunk_entry(&index, 16), None, "past the index");
    let ranges = window_ranges(&spec, &[0, 1], &[10..20, 40..50], &index, 1000);
    assert_eq!(ranges, vec![100..150]);
}
