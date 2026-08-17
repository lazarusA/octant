use crate::data::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;
use std::collections::HashMap;
use std::error::Error;
use zarrs::array::{Array, ArrayMetadata, DataType};
use zarrs::group::Group;
use zarrs::metadata::v3::MetadataV3;
use zarrs::metadata_ext::codec::blosc::{
    BloscCodecConfigurationNumcodecs, codec_blosc_v2_numcodecs_to_v3,
};
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::node::{NodeMetadata, NodePath, get_child_nodes};
use zarrs::storage::{
    ReadableStorageTraits, ReadableWritableListableStorage, ReadableWritableListableStorageTraits,
};

/// Normalizes Zarr v3 array metadata to handle non-standard / Python numcodecs codec representations
/// using `zarrs_metadata` and `zarrs_metadata_ext`.
pub fn normalize_v3_array_metadata(mut meta: serde_json::Value) -> serde_json::Value {
    let typesize = meta
        .get("data_type")
        .and_then(|d| serde_json::from_value::<MetadataV3>(d.clone()).ok())
        .and_then(|m| DataType::from_metadata(&m).ok())
        .map(|dt| dt.size());

    if let Some(codecs) = meta.get_mut("codecs").and_then(|c| c.as_array_mut()) {
        for codec in codecs {
            if let Some(obj) = codec.as_object_mut()
                && let Some(name) = obj.get("name").and_then(|n| n.as_str())
            {
                if name == "numcodecs.blosc" || name.ends_with(".blosc") {
                    if let Some(config) = obj.get("configuration").cloned()
                        && let Ok(blosc_numcodecs) =
                            serde_json::from_value::<BloscCodecConfigurationNumcodecs>(config)
                    {
                        let blosc_v3 = codec_blosc_v2_numcodecs_to_v3(&blosc_numcodecs, typesize);
                        obj.insert("name".to_string(), serde_json::json!("blosc"));
                        if let Ok(v3_config) = serde_json::to_value(&blosc_v3) {
                            obj.insert("configuration".to_string(), v3_config);
                        }
                    } else {
                        obj.insert("name".to_string(), serde_json::json!("blosc"));
                    }
                } else if let Some(suffix) = name.strip_prefix("numcodecs.") {
                    obj.insert("name".to_string(), serde_json::json!(suffix));
                }
            }
        }
    }
    meta
}

/// Helper to instantiate an Array from NodeMetadata, applying codec normalization if needed.
pub fn instantiate_array_from_node_metadata(
    store: ReadableWritableListableStorage,
    path: &str,
    node_meta: &NodeMetadata,
) -> Option<Array<dyn ReadableWritableListableStorageTraits>> {
    match node_meta {
        NodeMetadata::Array(array_meta) => {
            if let Ok(arr) = Array::new_with_metadata(store.clone(), path, array_meta.clone()) {
                Some(arr)
            } else if let Ok(meta_val) = serde_json::to_value(array_meta) {
                let norm_val = normalize_v3_array_metadata(meta_val);
                if let Ok(norm_meta) = serde_json::from_value::<ArrayMetadata>(norm_val) {
                    Array::new_with_metadata(store, path, norm_meta).ok()
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => Array::open(store, path).ok(),
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

    // 1. Try opening root as a Zarr Group
    if let Ok(group) = Group::open(store.clone(), "/") {
        // Check if group contains consolidated metadata (Zarr v2 or v3 inline)
        let consolidated_arrays = if let Some(ConsolidatedMetadata { metadata, .. }) =
            group.consolidated_metadata()
        {
            let mut found_vars = Vec::new();
            for (node_path, node_meta) in metadata {
                let clean_path = if node_path.starts_with('/') {
                    node_path.to_string()
                } else {
                    format!("/{}", node_path)
                };

                let array_opt =
                    instantiate_array_from_node_metadata(store.clone(), &clean_path, &node_meta);

                if let Some(array) = array_opt
                    && let Some(var_info) = variable_info_from_array(&array, node_path.as_str())
                {
                    found_vars.push(var_info);
                }
            }
            found_vars
        } else {
            Vec::new()
        };

        if !consolidated_arrays.is_empty() {
            return Ok(consolidated_arrays);
        }

        // 2. Discover child nodes recursively via zarrs get_child_nodes
        if let Ok(root_path) = NodePath::new("/") {
            discover_child_nodes_recursive(&store, &root_path, &mut variables);
        }
    } else if let Ok(array) = Array::open(store.clone(), "/") {
        // 3. Root itself is a single Array
        if let Some(var_info) = variable_info_from_array(&array, "data") {
            variables.push(var_info);
        }
    }

    // 4. Fallback: discover arrays via remote HTTP manifest inspection if base_url is present
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
            if let Ok(array) = Array::open(store.clone(), path_str) {
                if let Some(var_info) = variable_info_from_array(&array, var_name) {
                    variables.push(var_info);
                }
            } else if let Ok(child_path) = NodePath::new(path_str) {
                discover_child_nodes_recursive(store, &child_path, variables);
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

/// Constructs a VariableInfo struct from any ReadableStorageTraits zarrs Array.
pub fn variable_info_from_array<TStorage: ?Sized + ReadableStorageTraits>(
    array: &Array<TStorage>,
    var_name: &str,
) -> Option<VariableInfo> {
    let shape = array.shape().to_vec();
    if shape.is_empty() {
        return None;
    }

    let data_type = format!("{:?}", array.data_type());

    let dimension_names = array
        .dimension_names()
        .as_ref()
        .map(|names| {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| n.as_deref().unwrap_or(&format!("dim_{}", i)).to_string())
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
        .unwrap_or_else(|| default_dimension_names_for_rank(shape.len()));

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
