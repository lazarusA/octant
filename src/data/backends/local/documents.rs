//! The consolidated metadata built for local Zarr folders, from the
//! metadata documents `consolidate.rs` finds by walking them. Pure, so it is
//! tested on every target.

use crate::data::ome::parse::is_ome_attributes;
use serde_json::{Map, Value, json};

/// A v2 `.zmetadata` document of the v2 documents among `docs` (by key).
pub fn v2_document(docs: Map<String, Value>) -> Value {
    let metadata: Map<String, Value> = docs
        .into_iter()
        .filter(|(key, _)| !key.ends_with("zarr.json"))
        .collect();
    json!({ "zarr_consolidated_format": 1, "metadata": metadata })
}

/// Whether the v3 root `zarr.json` `root` needs consolidated metadata built:
/// a group without any that is no OME-Zarr store (whose discovery reads it
/// by known paths, so a large plate is never walked).
pub fn v3_needs_consolidation(root: &Value) -> bool {
    let group = root.get("node_type").and_then(Value::as_str) == Some("group");
    let consolidated = root
        .get("consolidated_metadata")
        .is_some_and(|c| !c.is_null());
    group && !consolidated && !is_ome_root(root.get("attributes"))
}

/// Writes the v3 nodes among `docs` (`{path}/zarr.json` keys) into the root
/// `zarr.json` `root` as its consolidated metadata.
pub fn consolidate_v3(root: &mut Value, docs: Map<String, Value>) {
    let nodes: Map<String, Value> = docs
        .into_iter()
        .filter_map(|(key, doc)| Some((key.strip_suffix("/zarr.json")?.to_owned(), doc)))
        .collect();
    root["consolidated_metadata"] =
        json!({ "kind": "inline", "must_understand": false, "metadata": nodes });
}

/// Whether a store's root attributes (v3 `attributes`, v2 `.zattrs`) are
/// OME-Zarr's.
pub fn is_ome_root(attributes: Option<&Value>) -> bool {
    attributes
        .and_then(Value::as_object)
        .is_some_and(is_ome_attributes)
}
