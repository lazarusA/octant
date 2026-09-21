//! Normalized Zarr array opening, candidate store key resolution, and array metadata inspection.

use std::error::Error;
use zarrs::array::{Array, ArrayMetadata};
use zarrs::node::NodeMetadata;
use zarrs::storage::ReadableStorageTraits;

use super::cf::ParsedCfAttributes;
pub use crate::data::codecs::normalize_v3_array_metadata;
use crate::data::metadata::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;

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

/// Resolves dimension names from array metadata, attributes (_ARRAY_DIMENSIONS), or default fallbacks.
pub fn resolve_array_dimension_names<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Vec<String> {
    let cf_attrs = ParsedCfAttributes::from_json_map(array.attributes());
    cf_attrs.resolve_dimension_names(array.dimension_names().as_deref(), array.shape().len())
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
    let cf_attrs = ParsedCfAttributes::from_json_map(array.attributes());
    let dimension_names =
        cf_attrs.resolve_dimension_names(array.dimension_names().as_deref(), shape.len());

    let zero_idx = vec![0u64; shape.len()];
    let chunk_shape = array
        .chunk_shape(&zero_idx)
        .ok()
        .map(|cs| cs.iter().map(|v| v.get()).collect::<Vec<u64>>())
        .unwrap_or_else(|| shape.clone());
    let file_size = calculate_variable_size_bytes(&shape, &data_type);

    Some(VariableInfo {
        name: var_name.to_string(),
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
    })
}
