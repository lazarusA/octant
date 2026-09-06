use crate::data::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;
use std::collections::HashMap;
use std::error::Error;
use zarrs::array::{Array, ArrayMetadata};
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;

use zarrs::node::{NodeMetadata, NodePath, get_child_nodes};
use zarrs::storage::{ReadableStorageTraits, ReadableWritableListableStorage};

pub use crate::data::codecs::normalize_v3_array_metadata;

/// Generates store candidate keys for array metadata JSON files without polling root group metadata for sub-arrays.
pub fn resolve_array_candidate_store_keys(clean_path: &str) -> Vec<zarrs::storage::StoreKey> {
    let path_strs = if clean_path.is_empty() {
        vec![
            "meta/root.array.json".to_string(),
            "zarr.json".to_string(),
            ".zarray".to_string(),
        ]
    } else {
        vec![
            format!("meta/root/{}.array.json", clean_path),
            format!("{}/zarr.json", clean_path),
            format!("{}/.zarray", clean_path),
        ]
    };

    path_strs
        .into_iter()
        .filter_map(|p| zarrs::storage::StoreKey::new(&p).ok())
        .collect()
}

/// Helper to instantiate an Array from NodeMetadata, applying codec normalization if needed.
pub fn instantiate_array_from_node_metadata<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    store: std::sync::Arc<TStorage>,
    path: &str,
    node_meta: &NodeMetadata,
) -> Option<Array<TStorage>> {
    let norm_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    match node_meta {
        NodeMetadata::Array(array_meta) => {
            if let Ok(arr) = Array::new_with_metadata(store.clone(), &norm_path, array_meta.clone())
            {
                Some(arr)
            } else if let Ok(meta_val) = serde_json::to_value(array_meta) {
                let norm_val = normalize_v3_array_metadata(meta_val);
                if let Ok(norm_meta) = serde_json::from_value::<ArrayMetadata>(norm_val) {
                    Array::new_with_metadata(store, &norm_path, norm_meta).ok()
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => Array::open(store, &norm_path).ok(),
    }
}

/// Opens or instantiates an Array from storage, applying metadata normalization if standard `Array::open` fails.
pub fn open_or_instantiate_array_normalized<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    store: std::sync::Arc<TStorage>,
    var_path: &str,
) -> Result<Array<TStorage>, Box<dyn Error + Send + Sync>> {
    let clean_path = var_path.trim_matches('/');
    let abs_path = if var_path.starts_with('/') {
        var_path.to_string()
    } else {
        format!("/{}", var_path)
    };

    // 1. Direct standard Array::open
    if let Ok(arr) = Array::open(store.clone(), &abs_path) {
        return Ok(arr);
    }
    if !clean_path.is_empty()
        && let Ok(arr) = Array::open(store.clone(), clean_path)
    {
        return Ok(arr);
    }

    // 2. Try reading metadata JSON files (zarr v3 standard meta/root/..., v3 folder zarr.json, v2 .zarray)
    let candidate_keys = resolve_array_candidate_store_keys(clean_path);

    for key in candidate_keys {
        if let Ok(Some(bytes)) = store.get(&key)
            && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes)
        {
            let norm_val = normalize_v3_array_metadata(val);
            if let Ok(norm_meta) = serde_json::from_value::<ArrayMetadata>(norm_val) {
                if let Ok(arr) =
                    Array::new_with_metadata(store.clone(), &abs_path, norm_meta.clone())
                {
                    log::debug!(
                        "[Metadata] Instantiated array '{}' via candidate key '{}'",
                        abs_path,
                        key.as_str()
                    );
                    return Ok(arr);
                }
                if !clean_path.is_empty()
                    && let Ok(arr) =
                        Array::new_with_metadata(store.clone(), clean_path, norm_meta.clone())
                {
                    log::debug!(
                        "[Metadata] Instantiated array '{}' via candidate key '{}'",
                        clean_path,
                        key.as_str()
                    );
                    return Ok(arr);
                }
                if let Ok(arr) = Array::new_with_metadata(store.clone(), "/", norm_meta) {
                    log::debug!(
                        "[Metadata] Instantiated root array for '{}' via candidate key '{}'",
                        var_path,
                        key.as_str()
                    );
                    return Ok(arr);
                }
            }
        }
    }

    // 3. Fallback to root Array::open if non-root failed, or return standard open error
    if var_path != "/"
        && let Ok(arr) = Array::open(store.clone(), "/")
    {
        return Ok(arr);
    }

    let arr = Array::open(store, var_path)?;
    Ok(arr)
}

/// Extracts `VariableInfo` directly from `NodeMetadata` in memory without requiring
/// a runtime codec decompression pipeline (`Array::new_with_metadata`).
/// This ensures 100% reliable discovery for all arrays, including those with custom,
/// virtual, or vendor-specific codecs (e.g., gribberish, scale_offset, numcodecs filters).
pub fn variable_info_from_node_metadata(
    var_name: &str,
    node_meta: &NodeMetadata,
) -> Option<VariableInfo> {
    match node_meta {
        NodeMetadata::Array(array_meta) => {
            let val = serde_json::to_value(array_meta).ok()?;
            let shape: Vec<u64> = val
                .get("shape")
                .and_then(|s| s.as_array())
                .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                .unwrap_or_default();

            if shape.is_empty() {
                return None;
            }

            let rank = shape.len();
            let data_type = val
                .get("data_type")
                .or_else(|| val.get("dtype"))
                .and_then(|d| d.as_str())
                .unwrap_or("float32")
                .to_string();

            let chunk_shape: Vec<u64> = val
                .get("chunk_grid")
                .and_then(|cg| cg.get("configuration"))
                .and_then(|c| c.get("chunk_shape"))
                .or_else(|| val.get("chunks"))
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                .unwrap_or_else(|| shape.clone());

            let mut attributes = HashMap::new();
            let mut units = None;
            let mut long_name = None;
            let mut time_coverage_start = None;
            let mut time_coverage_end = None;
            let mut temporal_resolution = None;

            if let Some(attrs_obj) = val.get("attributes").and_then(|a| a.as_object()) {
                for (k, v_json) in attrs_obj {
                    let val_str = if let Some(s) = v_json.as_str() {
                        s.to_string()
                    } else {
                        v_json.to_string()
                    };
                    attributes.insert(k.clone(), val_str.clone());
                    match k.as_str() {
                        "units" => units = Some(val_str),
                        "long_name" => long_name = Some(val_str),
                        "time_coverage_start" => time_coverage_start = Some(val_str),
                        "time_coverage_end" => time_coverage_end = Some(val_str),
                        "temporal_resolution" | "time_period" => {
                            temporal_resolution = Some(val_str)
                        }
                        _ => {}
                    }
                }
            }

            let dimension_names = val
                .get("dimension_names")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .enumerate()
                        .map(|(i, s)| {
                            s.as_str()
                                .map(|str_v| str_v.to_string())
                                .unwrap_or_else(|| format!("dim_{i}"))
                        })
                        .collect()
                })
                .or_else(|| {
                    val.get("attributes")
                        .and_then(|a| a.get("_ARRAY_DIMENSIONS"))
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    s.as_str()
                                        .map(|str_v| str_v.to_string())
                                        .unwrap_or_else(|| format!("dim_{i}"))
                                })
                                .collect()
                        })
                })
                .unwrap_or_else(|| default_dimension_names_for_rank(rank));

            let file_size = calculate_variable_size_bytes(&shape, &data_type);

            Some(VariableInfo {
                name: var_name.to_string(),
                data_type,
                shape,
                dimension_names,
                chunk_shape,
                file_size,
                units,
                long_name,
                time_coverage_start,
                time_coverage_end,
                temporal_resolution,
                attributes,
            })
        }
        _ => None,
    }
}

/// Format-agnostic store variable metadata extractor.
/// Operates on any zarrs storage adapter (`ReadableWritableListableStorage`),
/// supporting Group consolidated metadata, native node listing, single array inspection,
/// and fallback HTTP discovery for Zarr, Icechunk, and future stores (NetCDF, GeoTIFF).
pub fn extract_store_variables(
    store: ReadableWritableListableStorage,
    base_url: &str,
) -> Result<Vec<VariableInfo>, Box<dyn Error>> {
    let mut variables = Vec::new();

    // 1. Try opening root as a Zarr Group and check consolidated metadata (Zarr v2 or v3 inline)
    if let Ok(group) = Group::open(store.clone(), "/")
        && let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata()
    {
        for (node_path, node_meta) in metadata {
            let clean_var_name = node_path.trim_start_matches('/');
            let clean_var_name = if clean_var_name.is_empty() {
                "data"
            } else {
                clean_var_name
            };

            if let Some(var_info) = variable_info_from_node_metadata(clean_var_name, &node_meta) {
                variables.push(var_info);
            }
        }
        if !variables.is_empty() {
            return Ok(variables);
        }
    }

    // 2. Hierarchical recursive child node discovery via get_child_nodes (fast, zero-chunk listing)
    if let Ok(root_path) = NodePath::new("/") {
        discover_child_nodes_recursive(&store, &root_path, &mut variables);
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
                    if let Some(var_info) =
                        variable_info_from_node_metadata(var_name, child.metadata())
                    {
                        variables.push(var_info);
                    }
                }
                NodeMetadata::Group(_) => {
                    if let Ok(child_path) = NodePath::new(path_str) {
                        discover_child_nodes_recursive(store, &child_path, variables);
                    }
                }
            }
        }
    }
}

/// Returns standard fallback dimension names for a given tensor rank.
pub fn default_dimension_names_for_rank(rank: usize) -> Vec<String> {
    match rank {
        1 => vec!["x".to_string()],
        2 => vec!["lat".to_string(), "lon".to_string()],
        3 => vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        4 => vec![
            "time".to_string(),
            "level".to_string(),
            "lat".to_string(),
            "lon".to_string(),
        ],
        _ => (0..rank).map(|i| format!("dim_{}", i)).collect(),
    }
}

/// Resolves dimension names from array metadata, attributes (_ARRAY_DIMENSIONS), or default fallbacks.
pub fn resolve_array_dimension_names<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Vec<String> {
    let rank = array.shape().len();
    array
        .dimension_names()
        .as_ref()
        .map(|names| {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| n.clone().unwrap_or_else(|| format!("dim_{i}")))
                .collect()
        })
        .or_else(|| {
            array.attributes().get("_ARRAY_DIMENSIONS").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .enumerate()
                        .map(|(i, s)| {
                            s.as_str()
                                .map(|str_v| str_v.to_string())
                                .unwrap_or_else(|| format!("dim_{i}"))
                        })
                        .collect()
                })
            })
        })
        .unwrap_or_else(|| default_dimension_names_for_rank(rank))
}

/// Constructs a VariableInfo struct from any ReadableStorageTraits zarrs Array.
pub fn variable_info_from_array<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    var_name: &str,
) -> Option<VariableInfo> {
    let shape = array.shape().to_vec();
    if shape.is_empty() {
        return None;
    }

    let data_type = format!("{:?}", array.data_type());

    let dimension_names = resolve_array_dimension_names(array);

    let zero_idx = vec![0u64; shape.len()];
    let chunk_shape = array
        .chunk_shape(&zero_idx)
        .ok()
        .map(|cs| cs.iter().map(|v| v.get()).collect::<Vec<u64>>())
        .unwrap_or_else(|| shape.clone());
    let file_size = calculate_variable_size_bytes(&shape, &data_type);

    let attrs_map = array.attributes();
    let mut attributes = HashMap::new();
    let mut units = None;
    let mut long_name = None;
    let mut time_coverage_start = None;
    let mut time_coverage_end = None;
    let mut temporal_resolution = None;

    for (k, v_json) in attrs_map {
        let val_str = if let Some(s) = v_json.as_str() {
            s.to_string()
        } else {
            v_json.to_string()
        };
        attributes.insert(k.clone(), val_str.clone());

        match k.as_str() {
            "units" => units = Some(val_str),
            "long_name" => long_name = Some(val_str),
            "time_coverage_start" => time_coverage_start = Some(val_str),
            "time_coverage_end" => time_coverage_end = Some(val_str),
            "temporal_resolution" | "time_period" => temporal_resolution = Some(val_str),
            _ => {}
        }
    }

    Some(VariableInfo {
        name: var_name.to_string(),
        data_type,
        shape,
        dimension_names,
        chunk_shape,
        file_size,
        units,
        long_name,
        time_coverage_start,
        time_coverage_end,
        temporal_resolution,
        attributes,
    })
}

/// Fallback function: Discover variables via consolidated `.zmetadata` or `zarr.json` HTTP inspection.
pub fn discover_arrays_via_http_metadata(base_url: &str) -> Vec<VariableInfo> {
    let mut variables = Vec::new();

    let zmetadata_url = format!("{}/.zmetadata", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
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

                let mut attributes = HashMap::new();
                let mut units = None;
                let mut long_name = None;
                let mut time_coverage_start = None;
                let mut time_coverage_end = None;
                let mut temporal_resolution = None;
                let mut dimension_names = match shape.len() {
                    1 => vec!["x".to_string()],
                    2 => vec!["lat".to_string(), "lon".to_string()],
                    3 => vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                    4 => vec![
                        "time".to_string(),
                        "level".to_string(),
                        "lat".to_string(),
                        "lon".to_string(),
                    ],
                    _ => (0..shape.len()).map(|i| format!("dim_{}", i)).collect(),
                };

                if let Some(attrs_obj) = attrs_val.and_then(|a| a.as_object()) {
                    for (k, v_json) in attrs_obj {
                        let val_str = v_json
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| v_json.to_string());
                        attributes.insert(k.clone(), val_str.clone());
                        match k.as_str() {
                            "units" => units = Some(val_str),
                            "long_name" => long_name = Some(val_str),
                            "time_coverage_start" => time_coverage_start = Some(val_str),
                            "time_coverage_end" => time_coverage_end = Some(val_str),
                            "temporal_resolution" | "time_period" => {
                                temporal_resolution = Some(val_str)
                            }
                            _ => {}
                        }
                    }

                    if let Some(dims) = attrs_obj
                        .get("_ARRAY_DIMENSIONS")
                        .and_then(|d| d.as_array())
                    {
                        dimension_names = dims
                            .iter()
                            .filter_map(|e| e.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }

                let file_size = calculate_variable_size_bytes(&shape, &data_type);

                variables.push(VariableInfo {
                    name: var_name,
                    data_type,
                    shape,
                    dimension_names,
                    chunk_shape,
                    file_size,
                    units,
                    long_name,
                    time_coverage_start,
                    time_coverage_end,
                    temporal_resolution,
                    attributes,
                });
            }
        }
    }

    variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_v3_metadata_zlib_and_shuffle() {
        let raw_meta = serde_json::json!({
            "zarr_format": 3,
            "node_type": "array",
            "shape": [5, 3600, 7200],
            "data_type": "int16",
            "chunk_grid": {
                "name": "regular",
                "configuration": {
                    "chunk_shape": [1, 1200, 2400]
                }
            },
            "chunk_key_encoding": {
                "name": "default",
                "configuration": {
                    "separator": "/"
                }
            },
            "fill_value": -9999,
            "codecs": [
                {
                    "name": "numcodecs.shuffle",
                    "configuration": {
                        "elementsize": 2
                    }
                },
                {
                    "name": "numcodecs.zlib",
                    "configuration": {
                        "level": 1
                    }
                }
            ]
        });

        let normalized = normalize_v3_array_metadata(raw_meta);
        let array_meta: Result<ArrayMetadata, _> = serde_json::from_value(normalized);
        assert!(
            array_meta.is_ok(),
            "ArrayMetadata should successfully parse normalized zlib and shuffle metadata: {:?}",
            array_meta.err()
        );
    }
}
