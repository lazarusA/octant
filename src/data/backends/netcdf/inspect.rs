//! NetCDF variable metadata inspection and dimension coordinate scanning.

use std::collections::HashMap;
use std::path::Path;

use super::coord_scan::extract_dimension_coordinates;
use super::reader::{NcReader, NcVar};
use crate::data::metadata::{DatasetMetadata, VariableInfo};

fn variable_info(var: &NcVar, global_attrs: &HashMap<String, String>) -> VariableInfo {
    let attributes = var.attributes.clone();
    let shape: Vec<u64> = var.shape().map(|len| len as u64).collect();
    let data_type = var.data_type.to_string();
    let own_or_global = |key: &str| {
        (attributes.get(key))
            .or_else(|| global_attrs.get(key))
            .cloned()
    };
    let long_name = ["long_name", "standard_name", "description", "title"]
        .iter()
        .find_map(|key| attributes.get(*key).cloned());
    VariableInfo {
        name: var.path.clone(),
        file_size: crate::utils::units::calculate_variable_size_bytes(&shape, &data_type),
        data_type,
        chunk_shape: shape.clone(),
        shape,
        dimension_names: var.dims.iter().map(|(name, _)| name.clone()).collect(),
        units: attributes.get("units").cloned(),
        long_name,
        time_coverage_start: own_or_global("time_coverage_start"),
        time_coverage_end: own_or_global("time_coverage_end"),
        temporal_resolution: own_or_global("temporal_resolution"),
        attributes,
        ..VariableInfo::default()
    }
}

/// The metadata of the file `reader` reads, named by its title or else by `file_path`.
pub fn inspect_reader(reader: &dyn NcReader, file_path: &str) -> DatasetMetadata {
    let global_attrs = reader.global_attributes();
    let nc_vars = reader.variables();
    let mut variables: Vec<VariableInfo> = (nc_vars.iter())
        .map(|v| variable_info(v, &global_attrs))
        .collect();

    // CF grid mappings are scalar variables of the file itself.
    let mappings: HashMap<String, HashMap<String, String>> = (variables.iter())
        .filter(|v| v.attributes.contains_key("grid_mapping_name"))
        .map(|v| (v.name.clone(), v.attributes.clone()))
        .collect();
    crate::data::crs::attach_grid_mappings(&mut variables, |path| mappings.get(path).cloned());

    let dimension_coordinates = extract_dimension_coordinates(reader, &nc_vars, &variables);

    let dataset_name = global_attrs
        .get("title")
        .or_else(|| global_attrs.get("dataset_name"))
        .cloned()
        .unwrap_or_else(|| {
            Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("NetCDF Dataset")
                .to_string()
        });

    DatasetMetadata {
        name: dataset_name,
        store_type: "NetCDF".to_string(),
        variables,
        dimension_coordinates,
        ome: None,
    }
}
