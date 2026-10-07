//! Text coordinate reading: v3 `string` / `fixed_length_utf32`, v2 `<U` / `|O`, the label
//! cap, and the WASM preload ranges.

use std::sync::Arc;

use zarrs::array::{Array, ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

use super::strings::{MAX_LABELS, coordinate_preload_ranges, retrieve_array_as_strings};
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates;
use crate::utils::metadata::open_or_instantiate_array_normalized;

fn labels(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().map(|s| s.to_string()).collect())
}

/// A v3 `string` array of `values` split into chunks of `chunk` labels.
fn v3_strings(store: &Arc<MemoryStore>, path: &str, values: &[&str], chunk: u64) {
    let dt = DataType::from_metadata(&MetadataV3::new("string")).expect("string dtype");
    let array = ArrayBuilder::new(
        vec![values.len() as u64],
        vec![chunk],
        dt,
        FillValue::from(""),
    )
    .build(store.clone(), path)
    .expect("build string array");
    array.store_metadata().expect("store metadata");
    let subset = ArraySubset::new_with_shape(vec![values.len() as u64]);
    array
        .store_array_subset(&subset, values)
        .expect("store labels");
}

fn set(store: &MemoryStore, key: &str, bytes: Vec<u8>) {
    let key = StoreKey::new(key).expect("store key");
    store.set(&key, bytes.into()).expect("store bytes");
}

/// A v2 array written the way zarr-python 2 does, one uncompressed chunk.
fn v2_array(
    store: &MemoryStore,
    name: &str,
    dtype: &str,
    filters: &str,
    len: usize,
    chunk: Vec<u8>,
) {
    let zarray = format!(
        r#"{{"zarr_format": 2, "shape": [{len}], "chunks": [{len}], "dtype": "{dtype}",
            "compressor": null, "fill_value": null, "order": "C", "filters": {filters}}}"#
    );
    set(store, &format!("{name}/.zarray"), zarray.into_bytes());
    set(store, &format!("{name}/0"), chunk);
}

/// Opens like coordinate discovery does, normalizing metadata `zarrs` rejects as written.
fn open(store: &Arc<MemoryStore>, path: &str) -> Array<MemoryStore> {
    open_or_instantiate_array_normalized(store.clone(), path).expect("open array")
}

#[test]
fn v3_string_labels_are_read_whole_across_chunks() {
    let store = Arc::new(MemoryStore::new());
    let regions = ["Europe", " Africa ", "Asia", "Oceania", "Americas"];
    v3_strings(&store, "/region", &regions, 2);
    assert_eq!(
        retrieve_array_as_strings(&open(&store, "/region")),
        labels(&["Europe", "Africa", "Asia", "Oceania", "Americas"])
    );
}

#[test]
fn v3_fixed_length_utf32_labels_drop_their_padding() {
    let store = Arc::new(MemoryStore::new());
    let meta: MetadataV3 = serde_json::from_str(
        r#"{"name": "fixed_length_utf32", "configuration": {"length_bytes": 24}}"#,
    )
    .expect("utf32 metadata");
    let dt = DataType::from_metadata(&meta).expect("utf32 dtype");
    let fill = FillValue::from(vec![0u8; 24]);
    let array = ArrayBuilder::new(vec![2], vec![2], dt, fill)
        .build(store.clone(), "/member")
        .expect("build utf32 array");
    array.store_metadata().expect("store metadata");
    let members: [Vec<char>; 2] = ["r1i1p1", "r2"].map(|s| s.chars().collect());
    let subset = ArraySubset::new_with_shape(vec![2]);
    array
        .store_array_subset(&subset, &members)
        .expect("store members");
    assert_eq!(
        retrieve_array_as_strings(&open(&store, "/member")),
        labels(&["r1i1p1", "r2"])
    );
}

#[test]
fn v2_numpy_unicode_and_object_labels_are_read() {
    let store = Arc::new(MemoryStore::new());
    // `<U4`: four little-endian UTF-32 code units per label, NUL padded, written with the
    // `null` fill value xarray uses for string coordinates.
    let utf32: Vec<u8> = ["DAPI", "CD3"]
        .iter()
        .flat_map(|s| {
            let mut units: Vec<u32> = s.chars().map(u32::from).collect();
            units.resize(4, 0);
            units.into_iter().flat_map(u32::to_le_bytes)
        })
        .collect();
    v2_array(&store, "channel", "<U4", "null", 2, utf32);
    // `|O` with numcodecs vlen-utf8: item count, then each item's length and bytes.
    let mut vlen = 2u32.to_le_bytes().to_vec();
    for s in ["wet", "dry"] {
        vlen.extend((s.len() as u32).to_le_bytes());
        vlen.extend(s.as_bytes());
    }
    v2_array(&store, "season", "|O", r#"[{"id": "vlen-utf8"}]"#, 2, vlen);

    assert_eq!(
        retrieve_array_as_strings(&open(&store, "/channel")),
        labels(&["DAPI", "CD3"])
    );
    assert_eq!(
        retrieve_array_as_strings(&open(&store, "/season")),
        labels(&["wet", "dry"])
    );
}

#[test]
fn dimension_coordinates_keep_every_label_and_numeric_endpoints() {
    let store = Arc::new(MemoryStore::new());
    v3_strings(&store, "/region", &["Europe", "Africa", "Asia"], 3);
    let too_many: Vec<String> = (0..=MAX_LABELS).map(|i| format!("s{i}")).collect();
    let too_many: Vec<&str> = too_many.iter().map(String::as_str).collect();
    v3_strings(&store, "/station", &too_many, 1024);

    let dims = ["region", "station"].map(String::from);
    let coords = fetch_all_dimension_coordinates(store.clone(), &dims, Some("labels_store"));
    assert_eq!(
        coords
            .get("region")
            .and_then(|c| c.labels())
            .map(<[String]>::to_vec),
        labels(&["Europe", "Africa", "Asia"])
    );
    assert_eq!(
        coords.get("station"),
        None,
        "over the label cap stays unread"
    );
}

#[test]
#[allow(clippy::single_range_in_vec_init)]
fn wasm_preload_fetches_all_labels_but_only_numeric_endpoints() {
    let store = Arc::new(MemoryStore::new());
    v3_strings(&store, "/region", &["Europe", "Africa", "Asia"], 1);
    let labels = open(&store, "/region");
    assert_eq!(coordinate_preload_ranges(Some(&labels), 3), [0..3]);
    let none: Option<&Array<MemoryStore>> = None;
    assert_eq!(coordinate_preload_ranges(none, 10), [0..1, 9..10]);
    assert_eq!(coordinate_preload_ranges(none, 1), [0..1]);
    assert!(coordinate_preload_ranges(none, 0).is_empty());
}
