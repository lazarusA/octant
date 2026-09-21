//! Remote metadata inspection and variable extraction for WebAssembly Zarr datasets.

use super::WasmZarrBlockStore;
use crate::data::backends::http::fetch_url_bytes;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::metadata::{
    ParsedCfAttributes, open_or_instantiate_array_normalized, variable_info_from_array,
};
use crate::utils::units::calculate_variable_size_bytes;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;

/// Asynchronously inspects a remote Zarr URL in the browser and returns `DatasetMetadata`.
pub async fn inspect_wasm_remote_zarr(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim_end_matches('/');
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }

    let store = WasmZarrBlockStore::get_or_create(clean_url);

    // 1. Try fetching Zarr v2 consolidated `.zmetadata`
    let zmetadata_url = format!("{clean_url}/.zmetadata");
    if let Ok(zmetadata_bytes) = fetch_url_bytes(&zmetadata_url).await
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&zmetadata_bytes)
    {
        let _ = store.insert_key_bytes(".zmetadata", &zmetadata_bytes);

        if let Some(metadata_map) = val.get("metadata").and_then(|m| m.as_object()) {
            for (k, v) in metadata_map {
                if let Ok(json_str) = serde_json::to_string(v) {
                    let _ = store.insert_key_bytes(k, json_str.as_bytes());
                    let clean_k = k.trim_start_matches('/');
                    if clean_k != k {
                        let _ = store.insert_key_bytes(clean_k, json_str.as_bytes());
                    }
                }
            }
        }

        let mut variables = Vec::new();
        if let Some(metadata_obj) = val.get("metadata").and_then(|m| m.as_object()) {
            for (key, val) in metadata_obj {
                if key.ends_with("/.zarray") || key == ".zarray" || key.ends_with("/zarr.json") {
                    let var_name = key
                        .trim_end_matches("/.zarray")
                        .trim_end_matches("/zarr.json")
                        .trim_start_matches('/')
                        .to_string();
                    let var_name = if var_name.is_empty() {
                        "data".to_string()
                    } else {
                        var_name
                    };

                    let shape: Vec<u64> = val
                        .get("shape")
                        .and_then(|s| s.as_array())
                        .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                        .unwrap_or_default();

                    if shape.is_empty() {
                        continue;
                    }

                    let data_type = val
                        .get("dtype")
                        .or_else(|| val.get("data_type"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("float32")
                        .to_string();

                    let chunk_shape: Vec<u64> = val
                        .get("chunks")
                        .or_else(|| {
                            val.get("chunk_grid")
                                .and_then(|cg| cg.get("configuration"))
                                .and_then(|c| c.get("chunk_shape"))
                        })
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                        .unwrap_or_else(|| shape.clone());

                    let attrs_key = if key == ".zarray" {
                        ".zattrs".to_string()
                    } else {
                        format!("{var_name}/.zattrs")
                    };

                    let mut cf_attrs = metadata_obj
                        .get(&attrs_key)
                        .and_then(|a| a.as_object())
                        .map(ParsedCfAttributes::from_json_map)
                        .unwrap_or_default();

                    // Inherit ancestor group attributes (e.g. DGGS conventions)
                    for ancestor in crate::utils::path::ancestor_paths(&var_name) {
                        let p_val = if ancestor.is_empty() {
                            metadata_obj.get(".zattrs")
                        } else {
                            let parent_zattrs = format!("{ancestor}/.zattrs");
                            metadata_obj.get(&parent_zattrs)
                        };
                        if let Some(parent_val) = p_val {
                            let p_attrs_obj = parent_val
                                .get("attributes")
                                .and_then(|a| a.as_object())
                                .or_else(|| parent_val.as_object());
                            if let Some(p_obj) = p_attrs_obj {
                                let p_cf = ParsedCfAttributes::from_json_map(p_obj);
                                crate::utils::metadata::merge_parent_attributes(
                                    &mut cf_attrs.attributes,
                                    &p_cf.attributes,
                                );
                            }
                        }
                    }

                    let dimension_names = cf_attrs.resolve_dimension_names(None, shape.len());
                    let file_size = calculate_variable_size_bytes(&shape, &data_type);

                    variables.push(VariableInfo {
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
                    });
                }
            }
        }

        if !variables.is_empty() {
            return Ok(store.finalize_metadata(variables, clean_url).await);
        }
    }

    // 2. Try Zarr v3 `zarr.json` root
    let root_v3_url = format!("{clean_url}/zarr.json");
    if let Ok(v3_bytes) = fetch_url_bytes(&root_v3_url).await {
        let _ = store.insert_key_bytes("zarr.json", &v3_bytes);

        if let Ok(group) = Group::open(store.memory_store.clone(), "/") {
            let variables = if let Some(ConsolidatedMetadata { metadata, .. }) =
                group.consolidated_metadata()
            {
                for (node_path, node_meta) in &metadata {
                    let clean_path = node_path.trim_matches('/');
                    if let Ok(json_bytes) = serde_json::to_vec(node_meta) {
                        if clean_path.is_empty() {
                            let _ = store.insert_key_bytes("zarr.json", &json_bytes);
                        } else {
                            let _ = store
                                .insert_key_bytes(&format!("{clean_path}/zarr.json"), &json_bytes);
                            let _ = store.insert_key_bytes(
                                &format!("meta/root/{clean_path}.array.json"),
                                &json_bytes,
                            );
                            let _ = store.insert_key_bytes(
                                &format!("meta/root/{clean_path}.group.json"),
                                &json_bytes,
                            );
                        }
                    }
                }

                crate::utils::metadata::extract_store_variables_from_consolidated_metadata(
                    &metadata,
                )
            } else {
                Vec::new()
            };

            if !variables.is_empty() {
                return Ok(store.finalize_metadata(variables, clean_url).await);
            }
        }

        // Single root array case
        if let Ok(arr) = open_or_instantiate_array_normalized(store.memory_store.clone(), "/")
            && let Some(var_info) = variable_info_from_array(&arr, "data")
        {
            return Ok(store.finalize_metadata(vec![var_info], clean_url).await);
        }
    }

    // 3. Fallback: Unconsolidated OME-Zarr root .zattrs and .zgroup
    let root_zattrs_url = format!("{clean_url}/.zattrs");
    if let Ok(zattrs_bytes) = fetch_url_bytes(&root_zattrs_url).await
        && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&zattrs_bytes)
        && let Some(root_map) = v.as_object()
    {
        let _ = store.insert_key_bytes(".zattrs", &zattrs_bytes);
        let root_zgroup_url = format!("{clean_url}/.zgroup");
        if let Ok(zgroup_bytes) = fetch_url_bytes(&root_zgroup_url).await {
            let _ = store.insert_key_bytes(".zgroup", &zgroup_bytes);
        }

        let normalized = crate::utils::metadata::normalize_ngff_attributes(root_map.clone());
        if let Ok(root_zattrs) = serde_json::from_value::<crate::utils::metadata::ome::RootZattrs>(
            serde_json::Value::Object(normalized),
        ) && let Some(multiscale) = root_zattrs.multiscales.first()
        {
            for ds in &multiscale.datasets {
                let clean_path = ds.path.trim_matches('/');
                let zarray_url = format!("{clean_url}/{clean_path}/.zarray");
                if let Ok(zarray_bytes) = fetch_url_bytes(&zarray_url).await {
                    let _ = store.insert_key_bytes(&format!("{clean_path}/.zarray"), &zarray_bytes);
                }
            }

            let variables = crate::utils::metadata::extract_ome_multiscale_variables(
                store.memory_store.clone(),
                root_map,
                "",
            );

            if !variables.is_empty() {
                return Ok(store.finalize_metadata(variables, clean_url).await);
            }
        }
    }

    Err(format!(
        "Failed to inspect Zarr metadata at '{clean_url}'. Ensure the server allows CORS (Access-Control-Allow-Origin) and contains '.zmetadata', '.zattrs', or 'zarr.json'."
    ))
}
