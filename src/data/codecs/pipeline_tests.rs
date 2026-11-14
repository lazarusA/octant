//! Codec pipelines keep the codecs `zarrs` decodes: `numcodecs.pcodec` (the
//! ERA5 Icechunk arrays) and codecs Octant does not name are never replaced
//! by plain bytes, which would read compressed chunks as values.

use std::sync::Arc;

use serde_json::{Value, json};
use zarrs::array::{Array, ArrayMetadata, ArraySubset};
use zarrs::storage::store::MemoryStore;

use super::normalize_v3_array_metadata;

/// A float32 v3 array of `shape` in one chunk, with `codecs`.
fn metadata(shape: [u64; 2], codecs: Value) -> Value {
    json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": shape,
        "data_type": "float32",
        "chunk_grid": { "name": "regular", "configuration": { "chunk_shape": shape } },
        "chunk_key_encoding": { "name": "default", "configuration": { "separator": "/" } },
        "fill_value": 0.0,
        "codecs": codecs
    })
}

fn names(meta: &Value) -> Vec<&str> {
    meta["codecs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["name"].as_str())
        .collect()
}

#[test]
fn numcodecs_pcodec_is_kept_without_a_bytes_codec() {
    let meta = metadata(
        [4, 3],
        json!([{ "name": "numcodecs.pcodec", "configuration": { "level": 8 } }]),
    );
    let norm = normalize_v3_array_metadata(meta);
    assert_eq!(names(&norm), ["numcodecs.pcodec"]);
}

#[test]
fn unknown_codecs_are_kept_on_their_side_of_array_to_bytes() {
    let meta = metadata(
        [4, 3],
        json!([
            { "name": "example.before" },
            { "name": "bytes", "configuration": { "endian": "little" } },
            { "name": "example.after" }
        ]),
    );
    let norm = normalize_v3_array_metadata(meta);
    assert_eq!(names(&norm), ["example.before", "bytes", "example.after"]);
}

#[test]
fn an_unknown_codec_without_array_to_bytes_still_gets_bytes() {
    let meta = metadata([4, 3], json!([{ "name": "example.compressor" }]));
    let norm = normalize_v3_array_metadata(meta);
    assert_eq!(names(&norm), ["example.compressor", "bytes"]);
}

/// Values written through `numcodecs.pcodec` read back through the
/// normalized metadata, as the ERA5 Icechunk arrays are.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn pcodec_chunks_read_back_through_normalized_metadata() {
    let store = Arc::new(MemoryStore::new());
    let codecs = json!([{ "name": "numcodecs.pcodec", "configuration": { "level": 8 } }]);
    let meta: ArrayMetadata =
        serde_json::from_value(normalize_v3_array_metadata(metadata([4, 3], codecs)))
            .expect("pcodec metadata");
    let array = Array::new_with_metadata(store.clone(), "/a", meta).expect("pcodec array");
    array.store_metadata().expect("store metadata");
    let values: Vec<f32> = (0..12).map(|i| i as f32 * 0.5).collect();
    let subset = ArraySubset::new_with_shape(vec![4, 3]);
    array
        .store_array_subset(&subset, values.as_slice())
        .expect("store values");

    let opened = crate::utils::metadata::open_or_instantiate_array_normalized(store, "a")
        .expect("open array");
    let read = crate::data::backends::zarr::retrieve_array_subset_as_f32(
        &opened,
        crate::data::backends::zarr::ChunkReader::Direct,
        &subset,
    )
    .expect("read values");
    assert_eq!(read, values);
}

#[test]
fn gribberish_is_array_to_bytes() {
    let meta = metadata(
        [2, 4],
        json!([{ "name": "gribberish", "configuration": { "var": "PEVAP" } }]),
    );
    let norm = normalize_v3_array_metadata(meta);
    assert_eq!(names(&norm), ["gribberish"]);
}

/// A GRIB message stored as a chunk (as virtual stores built on GRIB files hold them)
/// reads back through the normalized metadata with gribberish's own values: a tiny
/// Lambert conformal RRFS field, complex packing, 5 of its 8 points missing.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn grib_message_chunks_read_back_through_normalized_metadata() {
    use zarrs::storage::{StoreKey, WritableStorageTraits};

    let bytes = crate::data::backends::grib::test_files::RRFS_PEVAP;
    let message = gribberish::message::read_message(bytes, 0).expect("message");
    let expected: Vec<f32> = (message.data().expect("data").into_iter())
        .map(|v| v as f32)
        .collect();

    let store = Arc::new(MemoryStore::new());
    let mut meta = metadata(
        [2, 4],
        json!([{ "name": "gribberish", "configuration": { "var": "PEVAP" } }]),
    );
    meta["data_type"] = json!("float64");
    meta["fill_value"] = json!("NaN");
    let meta: ArrayMetadata =
        serde_json::from_value(normalize_v3_array_metadata(meta)).expect("grib metadata");
    let array = Array::new_with_metadata(store.clone(), "/a", meta).expect("grib array");
    array.store_metadata().expect("store metadata");
    let key = StoreKey::new("a/c/0/0").expect("chunk key");
    store.set(&key, bytes.to_vec().into()).expect("store chunk");

    let opened = crate::utils::metadata::open_or_instantiate_array_normalized(store, "a")
        .expect("open array");
    let read = crate::data::backends::zarr::retrieve_array_subset_as_f32(
        &opened,
        crate::data::backends::zarr::ChunkReader::Direct,
        &ArraySubset::new_with_shape(vec![2, 4]),
    )
    .expect("read values");
    assert_eq!(read.len(), 8);
    assert_eq!(read.iter().filter(|v| v.is_nan()).count(), 5);
    for (r, e) in read.iter().zip(&expected) {
        assert!(r == e || (r.is_nan() && e.is_nan()), "{r} != {e}");
    }
}
