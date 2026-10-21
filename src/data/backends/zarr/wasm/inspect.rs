//! Remote metadata inspection and variable extraction for WebAssembly Zarr datasets.

use super::WasmZarrBlockStore;
use crate::data::backends::http::{fetch_url_bytes, is_missing};
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::metadata::{open_or_instantiate_array_normalized, variable_info_from_array};
use serde_json::Value;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;

/// Asynchronously inspects a remote Zarr URL in the browser and returns `DatasetMetadata`.
pub async fn inspect_wasm_remote_zarr(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim_end_matches('/');
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }

    let store = WasmZarrBlockStore::get_or_create(clean_url);
    // Consolidated metadata first, kept in memory so discovery reads it from
    // there: Zarr v2 `.zmetadata`, else the v3 root `zarr.json` (its
    // `consolidated_metadata` nodes written under their own keys).
    let consolidated = load_zmetadata(&store, clean_url).await;
    let format = match consolidated {
        Some(_) => Some(2),
        None => load_v3_root(&store, clean_url).await,
    };

    // OME-Zarr images, labels, wells and series (`crate::data::ome`)
    if let Some(found) = store.discover_ome(format).await {
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
        .map(super::consolidated::variables_from_consolidated_v2)
        .unwrap_or_default();
    // Zarr v3 `zarr.json` root, already in memory when the store has one
    if variables.is_empty() && store.has_key("zarr.json") {
        variables = variables_from_v3_root(&store);
    }
    if !variables.is_empty() {
        let reader = crate::data::ome::MetaReader::direct(store.memory_store.clone());
        crate::data::multiscale::geozarr::link_pyramids(&reader, &mut variables);
        crate::data::crs::attach_zarr_grid_mappings(&reader, &mut variables);
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

/// Fetches the root `zarr.json` into memory, with every node its
/// consolidated metadata lists. The store's Zarr format: `3` with a root
/// `zarr.json`, `2` without one, `None` when the request failed otherwise.
async fn load_v3_root(store: &WasmZarrBlockStore, clean_url: &str) -> Option<u8> {
    let bytes = match fetch_url_bytes(&format!("{clean_url}/zarr.json")).await {
        Ok(bytes) => bytes,
        Err(e) if is_missing(&e) => return Some(2),
        Err(_) => return None,
    };
    store.insert_key_bytes("zarr.json", &bytes).ok()?;
    if let Some(metadata) = v3_consolidated(store) {
        for (node_path, node_meta) in &metadata {
            if let Ok(json_bytes) = serde_json::to_vec(node_meta) {
                insert_v3_node(store, node_path.trim_matches('/'), &json_bytes);
            }
        }
    }
    Some(3)
}

/// The consolidated metadata of the v3 root in memory.
fn v3_consolidated(
    store: &WasmZarrBlockStore,
) -> Option<zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata> {
    let group = Group::open(store.memory_store.clone(), "/").ok()?;
    let ConsolidatedMetadata { metadata, .. } = group.consolidated_metadata()?;
    Some(metadata)
}

/// The variables of a Zarr v3 root: those its consolidated metadata lists,
/// else the root array itself.
fn variables_from_v3_root(store: &WasmZarrBlockStore) -> Vec<VariableInfo> {
    if let Some(metadata) = v3_consolidated(store) {
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
