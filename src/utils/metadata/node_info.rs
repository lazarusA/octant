//! Variable information extraction from NodeMetadata and consolidated metadata.

use std::collections::HashMap;
use zarrs::node::NodeMetadata;

use super::cf::{ParsedCfAttributes, merge_parent_attributes, resolve_ancestor_attributes};
use crate::data::metadata::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;

/// Extracts group attributes from group NodeMetadata.
pub fn extract_group_attributes_from_node_metadata(
    node_meta: &NodeMetadata,
) -> HashMap<String, String> {
    match node_meta {
        NodeMetadata::Group(group_meta) => {
            if let Ok(val) = serde_json::to_value(group_meta)
                && let Some(attrs_obj) = val.get("attributes").and_then(|a| a.as_object())
            {
                let cf = ParsedCfAttributes::from_json_map(attrs_obj);
                cf.attributes
            } else {
                HashMap::new()
            }
        }
        _ => HashMap::new(),
    }
}

/// Extracts `VariableInfo` directly from `NodeMetadata` in memory without requiring
/// a runtime codec decompression pipeline (`Array::new_with_metadata`).
pub fn variable_info_from_node_metadata(
    var_name: &str,
    node_meta: &NodeMetadata,
) -> Option<VariableInfo> {
    variable_info_from_node_metadata_with_parent_attributes(var_name, node_meta, None)
}

/// Extracts `VariableInfo` directly from `NodeMetadata` in memory with optional parent group attribute inheritance.
pub fn variable_info_from_node_metadata_with_parent_attributes(
    var_name: &str,
    node_meta: &NodeMetadata,
    parent_attributes: Option<&HashMap<String, String>>,
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

            let mut cf_attrs = val
                .get("attributes")
                .and_then(|a| a.as_object())
                .map(ParsedCfAttributes::from_json_map)
                .unwrap_or_default();

            // Inherit group-level attributes (e.g. DGGS conventions) if not overridden by the array
            if let Some(parent_attrs) = parent_attributes {
                merge_parent_attributes(&mut cf_attrs.attributes, parent_attrs);
            }

            let explicit_dim_names: Option<Vec<Option<String>>> = val
                .get("dimension_names")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|e| e.as_str().map(|s| s.to_string()))
                        .collect()
                });

            let dimension_names =
                cf_attrs.resolve_dimension_names(explicit_dim_names.as_deref(), rank);

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
        _ => None,
    }
}

/// Extracts variables from consolidated metadata map with group-to-array attribute inheritance.
pub fn extract_store_variables_from_consolidated_metadata(
    metadata: &HashMap<String, NodeMetadata>,
) -> Vec<VariableInfo> {
    // 1. First pass: collect all group attributes by group path
    let mut group_attrs: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (node_path, node_meta) in metadata {
        let clean_path = node_path.trim_matches('/');
        if matches!(node_meta, NodeMetadata::Group(_)) {
            let attrs = extract_group_attributes_from_node_metadata(node_meta);
            if !attrs.is_empty() {
                group_attrs.insert(clean_path.to_string(), attrs);
            }
        }
    }

    // 2. Second pass: extract array variables with inherited ancestor attributes
    let mut variables = Vec::new();
    for (node_path, node_meta) in metadata {
        let clean_path = node_path.trim_matches('/');
        let var_name = if clean_path.is_empty() {
            "data"
        } else {
            clean_path
        };

        if matches!(node_meta, NodeMetadata::Array(_)) {
            let inherited = resolve_ancestor_attributes(&group_attrs, clean_path);
            let parent_ref = if inherited.is_empty() {
                None
            } else {
                Some(&inherited)
            };

            if let Some(var_info) = variable_info_from_node_metadata_with_parent_attributes(
                var_name, node_meta, parent_ref,
            ) {
                variables.push(var_info);
            }
        }
    }

    variables
}
