use super::documents::{
    consolidate_v3, group_of, is_ome_root, v2_document, v3_needs_consolidation,
};
use serde_json::{Map, Value, json};
use std::sync::Arc;
use zarrs::group::Group;
use zarrs::storage::StoreKey;
use zarrs::storage::WritableStorageTraits;
use zarrs::storage::store::MemoryStore;

fn group() -> Value {
    json!({ "zarr_format": 3, "node_type": "group", "attributes": {} })
}

fn array() -> Value {
    json!({
        "zarr_format": 3, "node_type": "array", "shape": [4], "data_type": "float32",
        "chunk_grid": { "name": "regular", "configuration": { "chunk_shape": [4] } },
        "chunk_key_encoding": { "name": "default", "configuration": { "separator": "/" } },
        "fill_value": 0.0, "codecs": [{ "name": "bytes", "configuration": { "endian": "little" } }],
        "attributes": {}
    })
}

fn docs(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn a_built_v3_root_lists_every_node_to_zarrs() {
    let mut root = group();
    assert!(v3_needs_consolidation(&root));
    let found = docs(&[
        ("zarr.json", group()),
        ("t2m/zarr.json", array()),
        ("g/zarr.json", group()),
        ("g/v/zarr.json", array()),
    ]);
    consolidate_v3(&mut root, found);
    assert!(!v3_needs_consolidation(&root));

    let store = Arc::new(MemoryStore::new());
    let bytes = serde_json::to_vec(&root).expect("json");
    let key = StoreKey::new("zarr.json").expect("key");
    store.set(&key, bytes.into()).expect("set");
    let opened = Group::open(store, "/").expect("root opens");
    let metadata = opened
        .consolidated_metadata()
        .expect("consolidated")
        .metadata;
    let mut nodes: Vec<&str> = metadata.keys().map(String::as_str).collect();
    nodes.sort_unstable();
    assert_eq!(nodes, ["g", "g/v", "t2m"]);
    let variables =
        crate::utils::metadata::extract_store_variables_from_consolidated_metadata(&metadata);
    let mut names: Vec<&str> = variables.iter().map(|v| v.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, ["g/v", "t2m"]);
}

#[test]
fn consolidated_and_ome_roots_are_left_alone() {
    let mut nulled = group();
    nulled["consolidated_metadata"] = Value::Null;
    assert!(v3_needs_consolidation(&nulled));
    let mut consolidated = group();
    consolidate_v3(&mut consolidated, Map::new());
    assert!(!v3_needs_consolidation(&consolidated));
    let plate = json!({ "zarr_format": 3, "node_type": "group",
        "attributes": { "ome": { "version": "0.5", "plate": {} } } });
    assert!(!v3_needs_consolidation(&plate));
    assert!(!v3_needs_consolidation(&array()));
    assert!(is_ome_root(Some(&json!({ "plate": { "wells": [] } }))));
    assert!(!is_ome_root(Some(&json!({ "title": "ERA5" }))));
}

#[test]
fn a_v2_document_keeps_only_v2_keys() {
    let found = docs(&[
        (".zgroup", json!({ "zarr_format": 2 })),
        ("u/.zarray", json!({ "zarr_format": 2 })),
        ("u/.zattrs", json!({ "units": "m/s" })),
        ("w/zarr.json", array()),
    ]);
    let document = v2_document(found);
    assert_eq!(document["zarr_consolidated_format"], 1);
    let mut keys: Vec<&str> = document["metadata"]
        .as_object()
        .expect("metadata")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, [".zgroup", "u/.zarray", "u/.zattrs"]);
}

#[test]
fn groups_are_the_folders_of_group_documents() {
    let v2_group = json!({ "zarr_format": 2 });
    assert_eq!(group_of(".zgroup", &v2_group).as_deref(), Some(""));
    assert_eq!(group_of("a/b/.zgroup", &v2_group).as_deref(), Some("a/b"));
    assert_eq!(group_of("a/.zattrs", &v2_group), None);
    assert_eq!(group_of("a/x.zgroup", &v2_group), None);
    assert_eq!(group_of("zarr.json", &group()).as_deref(), Some(""));
    assert_eq!(group_of("g/zarr.json", &group()).as_deref(), Some("g"));
    assert_eq!(group_of("g/v/zarr.json", &array()), None);
}
