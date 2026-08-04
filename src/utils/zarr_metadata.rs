use crate::stores::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;
use std::collections::HashMap;
use std::error::Error;
use zarrs::array::Array;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::node::{NodePath, get_child_nodes};
use zarrs::storage::{ReadableStorageTraits, ReadableWritableListableStorage};

/// Extract all available variables natively using `zarrs` Group consolidated metadata API first,
/// falling back to native `zarrs::node::get_child_nodes` discovery if consolidated metadata is missing,
/// and preserving legacy `extract_store_variables` as an ultimate fallback.
pub fn extract_store_variables_consolidated(
    store: ReadableWritableListableStorage,
    base_url: &str,
) -> Result<Vec<VariableInfo>, Box<dyn Error>> {
    let mut variables = Vec::new();

    // 1. Try opening the root as a Zarr Group
    if let Ok(group) = Group::open(store.clone(), "/") {
        // Check if group contains consolidated metadata natively
        let consolidated_arrays = if let Some(ConsolidatedMetadata { metadata, .. }) =
            group.consolidated_metadata()
        {
            let mut found_vars = Vec::new();
            for (node_path, _node_meta) in metadata {
                let clean_path = if node_path.starts_with('/') {
                    node_path.to_string()
                } else {
                    format!("/{}", node_path)
                };
                if let Ok(array) = Array::open(store.clone(), &clean_path) {
                    if let Some(var_info) = variable_info_from_array(&array, node_path.as_str()) {
                        found_vars.push(var_info);
                    }
                }
            }
            found_vars
        } else {
            Vec::new()
        };

        if !consolidated_arrays.is_empty() {
            return Ok(consolidated_arrays);
        }

        // 2. Fallback: discover child nodes natively via zarrs get_child_nodes
        if let Ok(root_path) = NodePath::new("/") {
            if let Ok(children) = get_child_nodes(&store, &root_path, true) {
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
                    }
                }
            }
        }
    } else if let Ok(array) = Array::open(store.clone(), "/") {
        // 3. Root itself is a single Zarr Array
        if let Some(var_info) = variable_info_from_array(&array, "data") {
            variables.push(var_info);
        }
    }

    // 4. Fallback to existing extraction logic if native zarrs extraction found nothing
    if variables.is_empty() {
        return crate::utils::zarr::extract_store_variables(store, base_url);
    }

    Ok(variables)
}

/// Helper function to construct VariableInfo struct from a zarrs Array.
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
        .unwrap_or_else(|| match shape.len() {
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
        });

    let chunk_shape = shape.clone();
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
