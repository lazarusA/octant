//! Variables listed in a Zarr v2 consolidated `.zmetadata` document, read
//! without opening any array.

use crate::data::VariableInfo;
use crate::utils::metadata::ParsedCfAttributes;
use crate::utils::units::calculate_variable_size_bytes;
use serde_json::{Map, Value};

/// One variable per array listed in a v2 `.zmetadata` `metadata` object.
pub(super) fn variables_from_consolidated_v2(metadata: &Map<String, Value>) -> Vec<VariableInfo> {
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
    let var_name = variable_name(key);
    let (shape, data_type, chunk_shape) = array_layout(meta)?;
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
        multiscale: None,
        shard_shape: None,
    })
}

/// The variable an array metadata key names (`data` for the root array).
fn variable_name(key: &str) -> String {
    let name = key
        .trim_end_matches("/.zarray")
        .trim_end_matches("/zarr.json")
        .trim_start_matches('/');
    if name.is_empty() { "data" } else { name }.to_string()
}

/// An array's shape, data type and chunk shape; `None` for a scalar.
fn array_layout(meta: &Value) -> Option<(Vec<u64>, String, Vec<u64>)> {
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
    Some((shape, data_type, chunk_shape))
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
