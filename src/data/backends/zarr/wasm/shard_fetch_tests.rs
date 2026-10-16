//! Fetching shard parts over HTTP: the index and the window's inner chunks
//! of a whole-object shard (Zarr) and of a shard packed at a span of a
//! larger object (Icechunk), then the block decoded from them.

use std::collections::HashMap;

use zarrs::array::codec::ShardingIndexLocation;
use zarrs::storage::store::MemoryStore;

use super::WasmZarrBlockStore;
use super::range_server::RangeServer;
use super::shard_fetch::ShardRef;
use super::shard_plan::{merge, window_ranges};
use super::shard_tests::{source, source_at, spec_of, whole};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};
use crate::utils::executor::get_shared_tokio_rt;

/// Bytes packed before the shard in the Icechunk-like object.
const PAD: usize = 1000;

/// Rows 10..20, columns 40..50 of `v` (as `src` stores it) read from
/// `shard` (shard (0, 1)) into a fresh store named `name`; the bytes the
/// server sent.
fn read_window(
    server: &RangeServer,
    name: &str,
    shard: ShardRef,
    src: &MemoryStore,
) -> (Vec<f32>, u64) {
    let spec = spec_of(src);
    let store = WasmZarrBlockStore::get_or_create(&format!("{}/{name}", server.base));
    let meta = whole(src, "v/zarr.json");
    store
        .insert_key_bytes("v/zarr.json", &meta)
        .expect("metadata");
    let before = server.sent();
    let window = [10..20, 40..50];
    get_shared_tokio_rt()
        .block_on(store.fetch_shard_parts(&spec, &[shard], &window, &mut None))
        .expect("fetch");
    let sent = server.sent() - before;
    let reader =
        GenericZarrBlockStore::new(store.partial_store.clone(), "memory://", "zarr", "Zarr");
    let request = SliceRequest {
        variable: "v".into(),
        selections: vec![
            DimensionSelection::Range { start: 10, end: 20 },
            DimensionSelection::Range { start: 40, end: 50 },
        ],
    };
    let block = reader.fetch_block(&request).expect("block");
    (block.values.to_vec(), sent)
}

/// The bytes reading the window must fetch from `shard` (of `src`): its
/// index and the merged ranges of the window's inner chunks. Where zarrs
/// placed those chunks varies, so this is read from the shard's index.
fn planned(src: &MemoryStore, shard: &[u8]) -> u64 {
    let spec = spec_of(src);
    let range = spec.index_range(shard.len() as u64);
    let index = &shard[range.start as usize..range.end as usize];
    let window = [10..20, 40..50];
    let ranges = window_ranges(&spec, &[0, 1], &window, index, shard.len() as u64);
    let chunks: u64 = merge(ranges).iter().map(|r| r.end - r.start).sum();
    spec.index_len() + chunks
}

fn expected() -> Vec<f32> {
    (10..20)
        .flat_map(|r| (40..50).map(move |c| (r * 64 + c) as f32))
        .collect()
}

/// A server holding shard (0, 1) of [`source`] alone and packed at
/// `PAD..` of a larger object; the shard.
fn server_with_shard() -> (RangeServer, Vec<u8>) {
    let shard = whole(&source(), "v/c/0/1").to_vec();
    let mut packed = vec![0xAB; PAD];
    packed.extend_from_slice(&shard);
    packed.extend_from_slice(&[0xCD; 500]);
    let objects = HashMap::from([
        ("v/c/0/1".to_string(), shard.clone()),
        ("pack".to_string(), packed),
    ]);
    (RangeServer::start(objects), shard)
}

fn shard_ref(url: String, span: Option<std::ops::Range<u64>>) -> ShardRef {
    ShardRef {
        key: "v/c/0/1".into(),
        url,
        indices: vec![0, 1],
        span,
    }
}

#[test]
fn a_whole_object_shard_is_read_by_its_index_and_window_chunks() {
    let (server, shard) = server_with_shard();
    let url = format!("{}/v/c/0/1", server.base);
    let src = source();
    let (values, sent) = read_window(&server, "zarr", shard_ref(url, None), &src);
    assert_eq!(values, expected());
    assert_eq!(sent, planned(&src, &shard));
}

#[test]
fn a_packed_shard_is_read_within_its_span() {
    let (server, shard) = server_with_shard();
    let span = PAD as u64..(PAD + shard.len()) as u64;
    let url = format!("{}/pack", server.base);
    let src = source();
    let (values, sent) = read_window(&server, "icechunk", shard_ref(url, Some(span)), &src);
    assert_eq!(values, expected());
    assert_eq!(sent, planned(&src, &shard));
}

#[test]
fn a_missing_shard_is_empty() {
    let (server, _) = server_with_shard();
    let src = source();
    let store = WasmZarrBlockStore::get_or_create(&format!("{}/missing", server.base));
    let missing = shard_ref(format!("{}/v/c/9/9", server.base), None);
    let fetched = get_shared_tokio_rt().block_on(store.fetch_shard_parts(
        &spec_of(&src),
        &[missing],
        &[10..20, 40..50],
        &mut None,
    ));
    assert!(fetched.is_ok());
    assert_eq!(server.sent(), 0);
}

#[test]
fn a_short_index_is_an_error_not_an_empty_window() {
    // An object smaller than the shard index: no entry may read as empty.
    let objects = HashMap::from([("v/c/0/1".to_string(), vec![0u8; 100])]);
    let server = RangeServer::start(objects);
    let src = source();
    let store = WasmZarrBlockStore::get_or_create(&format!("{}/short", server.base));
    let shard = shard_ref(format!("{}/v/c/0/1", server.base), None);
    let fetched = get_shared_tokio_rt().block_on(store.fetch_shard_parts(
        &spec_of(&src),
        &[shard],
        &[10..20, 40..50],
        &mut None,
    ));
    assert!(fetched.is_err());
}

#[test]
fn an_index_at_the_start_is_read_by_its_size_and_a_range() {
    let src = source_at(ShardingIndexLocation::Start);
    let shard = whole(&src, "v/c/0/1").to_vec();
    let objects = HashMap::from([("v/c/0/1".to_string(), shard.clone())]);
    let server = RangeServer::start(objects);
    let url = format!("{}/v/c/0/1", server.base);
    let (values, sent) = read_window(&server, "start", shard_ref(url, None), &src);
    assert_eq!(values, expected());
    assert_eq!(sent, planned(&src, &shard));
}

#[test]
fn a_window_already_held_sends_nothing() {
    let (server, _) = server_with_shard();
    let src = source();
    let store = WasmZarrBlockStore::get_or_create(&format!("{}/again", server.base));
    let url = format!("{}/v/c/0/1", server.base);
    let (spec, shards) = (spec_of(&src), [shard_ref(url, None)]);
    let window = [10..20, 40..50];
    let fetch = || {
        get_shared_tokio_rt()
            .block_on(store.fetch_shard_parts(&spec, &shards, &window, &mut None))
            .expect("fetch");
    };
    fetch();
    // The size, the index and one merged range of the window's chunks.
    assert_eq!(server.requests(), 3);
    fetch();
    assert_eq!(server.requests(), 3, "held: no request");
}
