//! Store variable metadata discovery for hierarchical groups and consolidated metadata.

use std::collections::HashMap;
use std::error::Error;
use zarrs::array::Array;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::node::{NodeMetadata, NodePath, get_child_nodes};
use zarrs::storage::ReadableWritableListableStorage;

use super::array_open::variable_info_from_array;
#[cfg(not(target_arch = "wasm32"))]
use super::cf::ParsedCfAttributes;
use super::cf::merge_parent_attributes;
use super::node_info::{
    extract_group_attributes_from_node_metadata,
    extract_store_variables_from_consolidated_metadata,
    variable_info_from_node_metadata_with_parent_attributes,
};
use crate::data::metadata::VariableInfo;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::units::calculate_variable_size_bytes;

/// Format-agnostic store variable metadata extractor.
pub fn extract_store_variables(
    store: ReadableWritableListableStorage,
    base_url: &str,
) -> Result<Vec<VariableInfo>, Box<dyn Error>> {
    // 1. Try opening root as a Zarr Group and check consolidated metadata (Zarr v2 or v3 inline)
    if let Ok(group) = Group::open(store.clone(), "/")
        && let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata()
    {
        let variables = extract_store_variables_from_consolidated_metadata(&metadata);
        if !variables.is_empty() {
            return Ok(variables);
        }
    }

    // 2. Hierarchical recursive child node discovery via get_child_nodes (fast, zero-chunk listing)
    let mut variables = Vec::new();
    let root_attrs = Group::open(store.clone(), "/")
        .ok()
        .map(|g| {
            g.attributes()
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect::<HashMap<String, String>>()
        })
        .unwrap_or_default();

    if let Ok(root_path) = NodePath::new("/") {
        discover_child_nodes_recursive(&store, &root_path, &root_attrs, &mut variables);
    }
    if !variables.is_empty() {
        return Ok(variables);
    }

    // 3. Root itself is a single Array
    if let Ok(array) = Array::open(store.clone(), "/")
        && let Some(var_info) = variable_info_from_array(&array, "data")
    {
        variables.push(var_info);
        return Ok(variables);
    }

    // 4. Remote HTTP manifest inspection fallback if base_url is present
    if variables.is_empty() && !base_url.is_empty() {
        variables = discover_arrays_via_http_metadata(base_url);
    }

    Ok(variables)
}

fn discover_child_nodes_recursive(
    store: &ReadableWritableListableStorage,
    parent_path: &NodePath,
    parent_attrs: &HashMap<String, String>,
    variables: &mut Vec<VariableInfo>,
) {
    if let Ok(children) = get_child_nodes(store, parent_path, false) {
        for child in children {
            let path_str = child.path().as_str();
            let var_name = path_str.trim_start_matches('/');
            let var_name = if var_name.is_empty() {
                "data"
            } else {
                var_name
            };

            match child.metadata() {
                NodeMetadata::Array(_) => {
                    if let Some(var_info) = variable_info_from_node_metadata_with_parent_attributes(
                        var_name,
                        child.metadata(),
                        Some(parent_attrs),
                    ) {
                        variables.push(var_info);
                    }
                }
                NodeMetadata::Group(_) => {
                    let mut child_attrs = parent_attrs.clone();
                    let group_specific =
                        extract_group_attributes_from_node_metadata(child.metadata());
                    merge_parent_attributes(&mut child_attrs, &group_specific);

                    if let Ok(child_path) = NodePath::new(path_str) {
                        discover_child_nodes_recursive(store, &child_path, &child_attrs, variables);
                    }
                }
            }
        }
    }
}

/// Fallback function: Discover variables via consolidated `.zmetadata` or `zarr.json` HTTP inspection.
pub fn discover_arrays_via_http_metadata(base_url: &str) -> Vec<VariableInfo> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut variables = Vec::new();

        let zmetadata_url = format!("{}/.zmetadata", base_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .ok();

        let resp_opt = client
            .as_ref()
            .and_then(|c| c.get(&zmetadata_url).send().ok())
            .or_else(|| reqwest::blocking::get(&zmetadata_url).ok());

        if let Some(resp) = resp_opt
            && resp.status().is_success()
            && let Ok(bytes) = resp.bytes()
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && let Some(metadata_obj) = v.get("metadata").and_then(|m| m.as_object())
        {
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
                        .unwrap_or_else(|| vec![989, 72, 144]);

                    let chunk_shape: Vec<u64> = val
                        .get("chunks")
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                        .unwrap_or_else(|| shape.clone());

                    let data_type = val
                        .get("dtype")
                        .or_else(|| val.get("data_type"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("float32")
                        .to_string();

                    let zattrs_key = if var_name == "data" {
                        ".zattrs".to_string()
                    } else {
                        format!("{}/.zattrs", var_name)
                    };

                    let attrs_val = metadata_obj
                        .get(&zattrs_key)
                        .or_else(|| metadata_obj.get(".zattrs"));

                    let mut cf_attrs = attrs_val
                        .and_then(|a| a.as_object())
                        .map(ParsedCfAttributes::from_json_map)
                        .unwrap_or_default();

                    // Inherit ancestor group attributes (e.g. DGGS conventions)
                    for ancestor in crate::utils::path::ancestor_paths(&var_name) {
                        let p_val = if ancestor.is_empty() {
                            metadata_obj
                                .get(".zattrs")
                                .or_else(|| metadata_obj.get("zarr.json"))
                        } else {
                            let parent_zattrs = format!("{ancestor}/.zattrs");
                            let parent_zarr = format!("{ancestor}/zarr.json");
                            metadata_obj
                                .get(&parent_zattrs)
                                .or_else(|| metadata_obj.get(&parent_zarr))
                        };
                        if let Some(parent_val) = p_val {
                            let p_attrs_obj = parent_val
                                .get("attributes")
                                .and_then(|a| a.as_object())
                                .or_else(|| parent_val.as_object());
                            if let Some(p_obj) = p_attrs_obj {
                                let p_cf = ParsedCfAttributes::from_json_map(p_obj);
                                merge_parent_attributes(&mut cf_attrs.attributes, &p_cf.attributes);
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
        variables
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = base_url;
        Vec::new()
    }
}
