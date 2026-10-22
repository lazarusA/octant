//! Quantised geo-embeddings read dequantised: values times their pixel's
//! scale, a non-finite scale or an unwritten pixel read as missing.

use std::collections::HashMap;
use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

use super::generic::GenericZarrBlockStore;
use super::quantization::{dequantise, embedding_dim, scale_path, value_quantization};
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

/// The root attributes of a TESSERA-like store with 4-value embeddings.
fn root_attributes() -> serde_json::Value {
    serde_json::json!({
        "geoemb:dimensions": 4,
        "geoemb:quantization": {
            "method": "per_pixel_scale",
            "original_dtype": "float32",
            "quantized_dtype": "int8",
            "scale": {"type": "array", "array_name": "scales", "nodata": "NaN"}
        }
    })
}

fn attributes() -> HashMap<String, String> {
    let root = root_attributes();
    let map = root.as_object().expect("object");
    map.iter()
        .map(|(k, v)| (k.clone(), crate::utils::metadata::attr_string(v)))
        .collect()
}

fn put_group(store: &MemoryStore, key: &str, attributes: serde_json::Value) {
    let doc = serde_json::json!({"zarr_format": 3, "node_type": "group", "attributes": attributes});
    let key = StoreKey::new(key).expect("key");
    store
        .set(&key, doc.to_string().into_bytes().into())
        .expect("group");
}

/// `g/emb` (int8, `a` x `b` x `band` = 2 x 3 x 4) with only row `a = 0`
/// written, its `g/scales` (2 x 3, row 0 `[0.5, inf, 2]`, NaN fill), and
/// `g/flags` (int8, 2 x 3 x 2), which has no embedding dimension.
fn store() -> GenericZarrBlockStore {
    let store = Arc::new(MemoryStore::new());
    put_group(&store, "zarr.json", root_attributes());
    put_group(&store, "g/zarr.json", serde_json::json!({}));
    let int8 = DataType::from_metadata(&MetadataV3::new("int8")).expect("int8");
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");

    let mut emb = ArrayBuilder::new(
        vec![2, 3, 4],
        vec![1, 3, 4],
        int8.clone(),
        FillValue::from(0i8),
    );
    emb.dimension_names(Some(["a", "b", "band"]));
    let emb = emb.build(store.clone(), "/g/emb").expect("emb");
    emb.store_metadata().expect("emb metadata");
    let row: Vec<i8> = (1..=12).collect();
    emb.store_array_subset(&ArraySubset::new_with_ranges(&[0..1, 0..3, 0..4]), &row)
        .expect("emb row");

    let mut scales = ArrayBuilder::new(vec![2, 3], vec![1, 3], f32_dt, FillValue::from(f32::NAN));
    scales.dimension_names(Some(["a", "b"]));
    let scales = scales.build(store.clone(), "/g/scales").expect("scales");
    scales.store_metadata().expect("scales metadata");
    scales
        .store_array_subset(
            &ArraySubset::new_with_ranges(&[0..1, 0..3]),
            &[0.5f32, f32::INFINITY, 2.0],
        )
        .expect("scales row");

    let mut flags = ArrayBuilder::new(vec![2, 3, 2], vec![2, 3, 2], int8, FillValue::from(0i8));
    flags.dimension_names(Some(["a", "b", "c"]));
    let flags = flags.build(store.clone(), "/g/flags").expect("flags");
    flags.store_metadata().expect("flags metadata");
    flags
        .store_array_subset(&ArraySubset::new_with_shape(vec![2, 3, 2]), &[3i8; 12])
        .expect("flags");

    GenericZarrBlockStore::new(store, "memory://emb", "zarr", "Zarr")
}

fn whole(variable: &str, shape: &[usize]) -> SliceRequest {
    SliceRequest {
        variable: variable.into(),
        selections: shape
            .iter()
            .map(|&end| DimensionSelection::Range { start: 0, end })
            .collect(),
    }
}

#[test]
fn embeddings_read_as_values_times_their_pixel_scale() {
    let block = store()
        .fetch_block(&whole("g/emb", &[2, 3, 4]))
        .expect("block");
    let v = &block.values;
    // Pixel (0, 0): codes 1..=4 times 0.5.
    assert_eq!(&v[0..4], &[0.5, 1.0, 1.5, 2.0]);
    // Pixel (0, 1): an infinite scale is no data.
    assert!(v[4..8].iter().all(|x| x.is_nan()));
    // Pixel (0, 2): codes 9..=12 times 2.
    assert_eq!(&v[8..12], &[18.0, 20.0, 22.0, 24.0]);
    // Row 1 was never written: missing, not zeros.
    assert!(v[12..].iter().all(|x| x.is_nan()));
}

#[test]
fn arrays_without_the_embedding_dimension_stay_as_stored() {
    let block = store()
        .fetch_block(&whole("g/flags", &[2, 3, 2]))
        .expect("block");
    assert!(block.values.iter().all(|&x| x == 3.0));
}

#[test]
fn a_window_of_one_band_takes_its_pixels_scales() {
    let request = SliceRequest {
        variable: "g/emb".into(),
        selections: vec![
            DimensionSelection::Index(0),
            DimensionSelection::Range { start: 0, end: 3 },
            DimensionSelection::Index(2),
        ],
    };
    let block = store().fetch_block(&request).expect("block");
    assert_eq!(block.values[0], 1.5);
    assert!(block.values[1].is_nan());
    assert_eq!(block.values[2], 22.0);
}

#[test]
fn the_scale_array_is_the_sibling_named_by_the_convention() {
    let attrs = attributes();
    assert_eq!(
        scale_path(&attrs, "33N/embeddings", &[9, 10, 10, 4]),
        Some("33N/scales".into())
    );
    // No dimension of the embedding's length (a month mask), or the scale itself.
    assert_eq!(scale_path(&attrs, "33N/months", &[9, 10, 10, 12]), None);
    assert_eq!(scale_path(&attrs, "scales", &[4, 4]), None);
    let mut linear = attrs.clone();
    linear.insert(
        "geoemb:quantization".into(),
        r#"{"method": "linear", "scale": {"type": "scalar", "scale": 0.1}}"#.into(),
    );
    assert_eq!(scale_path(&linear, "33N/embeddings", &[9, 10, 10, 4]), None);
}

#[test]
fn the_embedding_dimension_leaves_the_scale_shape() {
    let attrs = attributes();
    assert_eq!(
        embedding_dim(&attrs, &[9, 10, 10, 4], &[9, 10, 10]),
        Some(3)
    );
    assert_eq!(
        embedding_dim(&attrs, &[9, 4, 10, 10], &[9, 10, 10]),
        Some(1)
    );
    assert_eq!(embedding_dim(&attrs, &[9, 10, 10, 4], &[9, 10, 11]), None);
}

#[test]
fn dequantise_scales_along_a_middle_dimension() {
    // Shape 2 x 2 x 3, dimension 1 the embedding: scales are 2 x 3.
    let mut values = vec![1.0f32; 12];
    let scales = [1.0, 2.0, 3.0, 4.0, f32::NAN, 6.0];
    dequantise(&mut values, &scales, &[2, 2, 3], 1);
    let expected = [
        1.0,
        2.0,
        3.0,
        1.0,
        2.0,
        3.0,
        4.0,
        f32::NAN,
        6.0,
        4.0,
        f32::NAN,
        6.0,
    ];
    for (v, e) in values.iter().zip(expected) {
        assert!(v == &e || (v.is_nan() && e.is_nan()), "{values:?}");
    }
}

#[test]
fn signed_square_and_linear_decode_each_code() {
    let mut attrs = attributes();
    attrs.insert(
        "geoemb:quantization".into(),
        r#"{"method": "signed_square", "nodata": -128}"#.into(),
    );
    let q = value_quantization(&attrs, &[2, 4, 3, 3]).expect("signed square");
    let mut v = [-128.0, 127.5, -63.75, 0.0];
    q.apply(None, (&[0], &[4]), &mut v).expect("apply");
    assert!(v[0].is_nan());
    assert_eq!(&v[1..], &[1.0, -0.25, 0.0]);

    attrs.insert(
        "geoemb:quantization".into(),
        r#"{"method": "linear", "scale": {"type": "scalar", "scale": 0.5, "offset": 1}}"#.into(),
    );
    let q = value_quantization(&attrs, &[2, 4, 3, 3]).expect("linear");
    let mut v = [2.0, -4.0];
    q.apply(None, (&[0], &[2]), &mut v).expect("apply");
    assert_eq!(v, [2.0, -1.0]);
    // A band coordinate (1D) and the per-pixel method are not decoded per code.
    assert_eq!(value_quantization(&attrs, &[4]), None);
    assert_eq!(value_quantization(&attributes(), &[2, 4, 3, 3]), None);
}
