//! Numeric coordinate streaming: even spacing across chunks, uneven values, `f64`
//! precision, sparse chunks and failed reads.

use std::sync::Arc;

use zarrs::array::{Array, ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

use super::coords::read_coordinate;
use crate::data::CoordValues;

/// A v3 array at `/{name}` of `dtype` with `len` values in chunks of `chunk`.
fn array(
    store: &Arc<MemoryStore>,
    name: &str,
    dtype: &str,
    len: u64,
    chunk: u64,
) -> Array<MemoryStore> {
    let dt = DataType::from_metadata(&MetadataV3::new(dtype)).expect("data type");
    let fill = match dtype {
        "float64" => FillValue::from(f64::NAN),
        "float32" => FillValue::from(f32::NAN),
        _ => FillValue::from(0i64),
    };
    let array = ArrayBuilder::new(vec![len], vec![chunk], dt, fill)
        .build(store.clone(), &format!("/{name}"))
        .expect("build array");
    array.store_metadata().expect("store metadata");
    array
}

fn f64_axis(name: &str, values: &[f64], chunk: u64) -> (Arc<MemoryStore>, Array<MemoryStore>) {
    let store = Arc::new(MemoryStore::new());
    let a = array(&store, name, "float64", values.len() as u64, chunk);
    let subset = ArraySubset::new_with_shape(vec![values.len() as u64]);
    a.store_array_subset(&subset, values).expect("store values");
    (store, a)
}

#[test]
fn evenly_spaced_chunks_stream_into_start_and_step() {
    let lon: Vec<f64> = (0..1440).map(|i| -180.0 + 0.25 * f64::from(i)).collect();
    let (_store, a) = f64_axis("lon", &lon, 100);
    assert_eq!(
        read_coordinate(&a),
        Some(CoordValues::Regular {
            start: -180.0,
            step: 0.25,
            len: 1440
        })
    );
}

#[test]
fn spacing_that_breaks_in_a_later_chunk_keeps_every_value() {
    // Even for three chunks, then the levels thicken: the earlier values are regenerated.
    let mut depth: Vec<f64> = (0..6).map(|i| 10.0 * f64::from(i)).collect();
    depth.extend([75.0, 110.0, 200.0]);
    let (_store, a) = f64_axis("depth", &depth, 2);
    let coords = read_coordinate(&a).expect("depth");
    assert!(matches!(coords, CoordValues::Values(_)));
    let read: Vec<_> = (0..depth.len()).filter_map(|i| coords.number(i)).collect();
    assert_eq!(read, depth);
}

#[test]
fn epoch_seconds_keep_full_precision() {
    // 1.7e9 seconds is beyond f32's precision (steps of 128 s there).
    let store = Arc::new(MemoryStore::new());
    let t0 = 1_700_000_000i64;
    let seconds = [t0, t0 + 1, t0 + 3, t0 + 4];
    let a = array(&store, "time", "int64", 4, 4);
    a.store_array_subset(&ArraySubset::new_with_shape(vec![4]), &seconds)
        .expect("store seconds");
    let coords = read_coordinate(&a).expect("time");
    assert_eq!(coords.number(2), Some((t0 + 3) as f64));
    assert!(
        matches!(coords, CoordValues::Values(_)),
        "1 s then 2 s steps are uneven"
    );
}

#[test]
fn f32_axes_tolerate_their_rounding() {
    let store = Arc::new(MemoryStore::new());
    let lat: Vec<f32> = (0..1801)
        .map(|j| (90.0 - 0.1 * f64::from(j)) as f32)
        .collect();
    let a = array(&store, "lat", "float32", 1801, 500);
    a.store_array_subset(&ArraySubset::new_with_shape(vec![1801]), &lat)
        .expect("store lat");
    let coords = read_coordinate(&a).expect("lat");
    assert!(matches!(coords, CoordValues::Regular { len: 1801, .. }));
    let south = coords.last_number().unwrap_or(f64::NAN);
    assert!((south + 90.0).abs() < 1e-4);
}

#[test]
#[allow(clippy::single_range_in_vec_init)]
fn a_missing_chunk_reads_as_fill_not_an_error() {
    let store = Arc::new(MemoryStore::new());
    let a = array(&store, "x", "float64", 6, 2);
    a.store_array_subset(&ArraySubset::new_with_ranges(&[0..2]), &[0.0, 1.0])
        .expect("first chunk");
    a.store_array_subset(&ArraySubset::new_with_ranges(&[4..6]), &[4.0, 5.0])
        .expect("last chunk");
    let coords = read_coordinate(&a).expect("x");
    assert_eq!(coords.len(), 6);
    assert!(coords.number(2).is_some_and(f64::is_nan));
}

#[test]
fn a_corrupt_middle_chunk_falls_back_to_the_endpoints() {
    let values = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let (store, a) = f64_axis("x", &values, 2);
    let key = StoreKey::new("x/c/1").expect("chunk key");
    store
        .set(&key, vec![1u8, 2, 3].into())
        .expect("corrupt chunk");
    assert_eq!(
        read_coordinate(&a),
        Some(CoordValues::Endpoints {
            first: 0.0,
            last: 5.0,
            len: 6
        })
    );
}
