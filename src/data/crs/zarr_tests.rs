//! CRS keys read from Zarr stores: scalar grid mapping arrays, root and
//! ancestor groups.

use std::sync::Arc;

use super::tests::UTM36N_WKT1;
use super::{CrsRef, assign, discover, named};
use crate::data::metadata::VariableInfo;

/// A Zarr store whose band names a scalar `spatial_ref` grid mapping, as
/// rioxarray writes them, and whose root group names its CRS (`proj:code`,
/// inherited by a band without a mapping), inspected as a dataset.
#[test]
fn zarr_grid_mappings_are_read_from_scalar_arrays() {
    use crate::data::backends::zarr::GenericZarrBlockStore;
    use crate::data::blocks::BlockStore;
    use zarrs::storage::store::MemoryStore;
    use zarrs::storage::{StoreKey, WritableStorageTraits};

    let store = Arc::new(MemoryStore::new());
    let put = |key: &str, value: serde_json::Value| {
        let key = StoreKey::new(key).expect("key");
        let bytes = serde_json::to_vec(&value).expect("json");
        store.set(&key, bytes.into()).expect("set");
    };
    let array = |shape: &[u64], attributes: serde_json::Value| {
        serde_json::json!({
            "zarr_format": 3, "node_type": "array", "shape": shape,
            "data_type": "float32", "fill_value": 0.0,
            "chunk_grid": {"name": "regular", "configuration": {"chunk_shape": shape}},
            "chunk_key_encoding": {"name": "default", "configuration": {"separator": "/"}},
            "codecs": [{"name": "bytes", "configuration": {"endian": "little"}}],
            "attributes": attributes,
            "dimension_names": if shape.is_empty() { serde_json::json!([]) } else { serde_json::json!(["y", "x"]) },
        })
    };
    put(
        "zarr.json",
        serde_json::json!({"zarr_format": 3, "node_type": "group",
            "attributes": {"proj:code": "EPSG:32636"}}),
    );
    put("b03/zarr.json", array(&[2, 2], serde_json::json!({})));
    put(
        "b02/zarr.json",
        array(&[2, 2], serde_json::json!({"grid_mapping": "spatial_ref"})),
    );
    put(
        "spatial_ref/zarr.json",
        array(&[], serde_json::json!({"crs_wkt": UTM36N_WKT1})),
    );

    let zarr = GenericZarrBlockStore::new(store.clone(), "memory://crs", "zarr", "Zarr");
    let mut meta = zarr.inspect().expect("inspect");
    assign(&mut meta);
    let b02 = meta
        .variables
        .iter()
        .find(|v| v.leaf_name() == "b02")
        .expect("b02");
    assert_eq!(b02.crs, Some(CrsRef::Epsg(32636)));
    assert_eq!(named(b02), Some("WGS 84 / UTM zone 36N"));
    let b03 = meta.variables.iter().find(|v| v.leaf_name() == "b03");
    assert_eq!(b03.and_then(|v| v.crs.clone()), Some(CrsRef::Epsg(32636)));
}

/// Variables built from their arrays alone (the web's Icechunk) take their
/// groups' CRS keys, the nearest group's first.
#[test]
fn group_crs_keys_are_inherited_nearest_first() {
    use zarrs::storage::store::MemoryStore;
    use zarrs::storage::{StoreKey, WritableStorageTraits};
    let store = Arc::new(MemoryStore::new());
    let group = |key: &str, attrs: serde_json::Value| {
        let doc = serde_json::json!({"zarr_format": 3, "node_type": "group", "attributes": attrs});
        let bytes = serde_json::to_vec(&doc).expect("json");
        store
            .set(&StoreKey::new(key).expect("key"), bytes.into())
            .expect("set");
    };
    group("zarr.json", serde_json::json!({"proj:code": "EPSG:4326"}));
    group(
        "r10m/zarr.json",
        serde_json::json!({"proj:code": "EPSG:32636"}),
    );
    let var = |name: &str| VariableInfo {
        name: name.into(),
        ..Default::default()
    };
    let mut vars = vec![var("r10m/b02"), var("b03")];
    let reader = crate::data::ome::MetaReader::direct(store);
    super::inherit_zarr_group_keys(&reader, &mut vars);
    let crs = |v: &VariableInfo| discover(&v.attributes, &v.dimension_names);
    assert_eq!(crs(&vars[0]), Some(CrsRef::Epsg(32636)), "its group's");
    assert_eq!(crs(&vars[1]), Some(CrsRef::Epsg(4326)), "the root's");
}
