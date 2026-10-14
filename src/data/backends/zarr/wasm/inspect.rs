//! Remote metadata inspection and variable extraction for WebAssembly Zarr datasets.

use super::WasmZarrBlockStore;
use crate::data::backends::http::fetch_url_bytes;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::metadata::{
    ParsedCfAttributes, open_or_instantiate_array_normalized, variable_info_from_array,
};
use crate::utils::units::calculate_variable_size_bytes;
use serde_json::{Map, Value};
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;

/// Asynchronously inspects a remote Zarr URL in the browser and returns `DatasetMetadata`.
pub async fn inspect_wasm_remote_zarr(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim_end_matches('/');
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }

    let store = WasmZarrBlockStore::get_or_create(clean_url);
    // 1. Zarr v2 consolidated `.zmetadata`, kept in memory for the steps below
    let consolidated = load_zmetadata(&store, clean_url).await;

    // OME-Zarr images, labels, wells and series (`crate::data::ome`)
    if let Some(found) = store.discover_ome().await {
        crate::data::ome::report_warnings(&found, clean_url);
        let ome = Some(std::sync::Arc::new(found.dataset));
        return Ok(store
            .finalize_metadata(found.variables, ome, clean_url)
            .await);
    }

    let mut variables = consolidated
        .as_ref()
        .and_then(|val| val.get("metadata"))
        .and_then(|m| m.as_object())
        .map(variables_from_consolidated_v2)
        .unwrap_or_default();
    // 2. Zarr v3 `zarr.json` root (the OME-Zarr discovery above already
    // fetched it into memory when the store has one)
    if variables.is_empty() && store.has_key("zarr.json") {
        variables = variables_from_v3_root(&store);
    }
    if !variables.is_empty() {
        return Ok(store.finalize_metadata(variables, None, clean_url).await);
    }

    Err(format!(
        "Failed to inspect Zarr metadata at '{clean_url}'. Ensure the server allows CORS (Access-Control-Allow-Origin) and contains '.zmetadata', '.zattrs', or 'zarr.json'."
    ))
}

/// The store's `.zmetadata` document, with every key it lists written into
/// the memory store.
async fn load_zmetadata(store: &WasmZarrBlockStore, clean_url: &str) -> Option<Value> {
    let bytes = fetch_url_bytes(&format!("{clean_url}/.zmetadata"))
        .await
        .ok()?;
    let val = serde_json::from_slice::<Value>(&bytes).ok()?;
    let _ = store.insert_key_bytes(".zmetadata", &bytes);
    for (k, v) in val.get("metadata").and_then(|m| m.as_object())? {
        if let Ok(json_str) = serde_json::to_string(v) {
            let _ = store.insert_key_bytes(k, json_str.as_bytes());
            let clean_k = k.trim_start_matches('/');
            if clean_k != k {
                let _ = store.insert_key_bytes(clean_k, json_str.as_bytes());
            }
        }
    }
    Some(val)
}

/// One variable per array listed in a v2 `.zmetadata` `metadata` object.
fn variables_from_consolidated_v2(metadata: &Map<String, Value>) -> Vec<VariableInfo> {
    metadata
        .iter()
        .filter(|(key, _)| {
            key.ends_with("/.zarray") || *key == ".zarray" || key.ends_with("/zarr.json")
        })
        .filter_map(|(key, meta)| consolidated_variable(metadata, key, meta))
        .collect()
}

/// The array `meta` stored under `key`, with its own and its ancestors'
/// attributes; `None` for a scalar.
fn consolidated_variable(
    metadata: &Map<String, Value>,
    key: &str,
    meta: &Value,
) -> Option<VariableInfo> {
    let var_name = key
        .trim_end_matches("/.zarray")
        .trim_end_matches("/zarr.json")
        .trim_start_matches('/');
    let var_name = if var_name.is_empty() {
        "data"
    } else {
        var_name
    }
    .to_string();
    let u64s = |v: &Value| -> Option<Vec<u64>> {
        Some(v.as_array()?.iter().filter_map(Value::as_u64).collect())
    };
    let shape = meta.get("shape").and_then(u64s).unwrap_or_default();
    if shape.is_empty() {
        return None;
    }
    let data_type = meta
        .get("dtype")
        .or_else(|| meta.get("data_type"))
        .and_then(|d| d.as_str())
        .unwrap_or("float32")
        .to_string();
    let chunk_shape = meta
        .get("chunks")
        .or_else(|| meta.pointer("/chunk_grid/configuration/chunk_shape"))
        .and_then(u64s)
        .unwrap_or_else(|| shape.clone());
    let attrs_key = if key == ".zarray" {
        ".zattrs".to_string()
    } else {
        format!("{var_name}/.zattrs")
    };
    let mut cf_attrs = metadata
        .get(&attrs_key)
        .and_then(|a| a.as_object())
        .map(ParsedCfAttributes::from_json_map)
        .unwrap_or_default();
    inherit_ancestor_attributes(metadata, &var_name, &mut cf_attrs);
    let dimension_names = cf_attrs.resolve_dimension_names(None, shape.len());
    let file_size = calculate_variable_size_bytes(&shape, &data_type);
    Some(VariableInfo {
        name: var_name,
        data_type,
        shape,
        dimension_names,
        chunk_shape,
        file_size,
        units: cf_attrs.units,
        long_name: cf_attrs.long_name,
        time_coverage_start: cf_attrs.time_coverage_start,
        time_coverage_end: cf_attrs.time_coverage_end,
        temporal_resolution: cf_attrs.temporal_resolution,
        attributes: cf_attrs.attributes,
        ome: None,
    })
}

/// Merges the attributes of `var_name`'s ancestor groups (e.g. DGGS
/// conventions) into `cf_attrs`.
fn inherit_ancestor_attributes(
    metadata: &Map<String, Value>,
    var_name: &str,
    cf_attrs: &mut ParsedCfAttributes,
) {
    for ancestor in crate::utils::path::ancestor_paths(var_name) {
        let parent = if ancestor.is_empty() {
            metadata.get(".zattrs")
        } else {
            metadata.get(&format!("{ancestor}/.zattrs"))
        };
        let Some(parent) = parent else { continue };
        let attrs = parent.get("attributes").and_then(|a| a.as_object());
        if let Some(p_obj) = attrs.or_else(|| parent.as_object()) {
            let p_cf = ParsedCfAttributes::from_json_map(p_obj);
            crate::utils::metadata::merge_parent_attributes(
                &mut cf_attrs.attributes,
                &p_cf.attributes,
            );
        }
    }
}

/// The variables of a Zarr v3 root: those its consolidated metadata lists
/// (written into the memory store), else the root array itself.
fn variables_from_v3_root(store: &WasmZarrBlockStore) -> Vec<VariableInfo> {
    if let Ok(group) = Group::open(store.memory_store.clone(), "/")
        && let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata()
    {
        for (node_path, node_meta) in &metadata {
            if let Ok(json_bytes) = serde_json::to_vec(node_meta) {
                insert_v3_node(store, node_path.trim_matches('/'), &json_bytes);
            }
        }
        let variables =
            crate::utils::metadata::extract_store_variables_from_consolidated_metadata(&metadata);
        if !variables.is_empty() {
            return variables;
        }
    }
    // Single root array case
    open_or_instantiate_array_normalized(store.memory_store.clone(), "/")
        .ok()
        .and_then(|arr| variable_info_from_array(&arr, "data"))
        .into_iter()
        .collect()
}

/// Writes a consolidated v3 node's metadata under the keys it may be read from.
fn insert_v3_node(store: &WasmZarrBlockStore, path: &str, json_bytes: &[u8]) {
    if path.is_empty() {
        let _ = store.insert_key_bytes("zarr.json", json_bytes);
        return;
    }
    let _ = store.insert_key_bytes(&format!("{path}/zarr.json"), json_bytes);
    let _ = store.insert_key_bytes(&format!("meta/root/{path}.array.json"), json_bytes);
    let _ = store.insert_key_bytes(&format!("meta/root/{path}.group.json"), json_bytes);
}
