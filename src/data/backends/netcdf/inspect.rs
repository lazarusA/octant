//! NetCDF variable metadata inspection and dimension coordinate scanning.

use std::collections::HashMap;
use std::path::Path;

use super::attrs::{extract_global_attributes, extract_variable_attributes, var_type_to_string};
use super::coord_scan::extract_dimension_coordinates;
use crate::data::blocks::BlockStoreError;
use crate::data::metadata::{DatasetMetadata, VariableInfo};

pub fn collect_group_variables(
    group: &netcdf::Group,
    prefix: &str,
    global_attrs: &HashMap<String, String>,
    variables: &mut Vec<VariableInfo>,
) {
    for var in group.variables() {
        let name = if prefix.is_empty() {
            var.name()
        } else {
            format!("{prefix}/{}", var.name())
        };
        let vartype = var.vartype();
        let data_type = var_type_to_string(&vartype).to_string();

        let dims = var.dimensions();
        let shape: Vec<u64> = dims.iter().map(|d| d.len() as u64).collect();
        let dimension_names: Vec<String> = dims.iter().map(|d| d.name()).collect();
        let chunk_shape = shape.clone();

        let attributes = extract_variable_attributes(&var);

        let units = attributes.get("units").cloned();
        let long_name = attributes
            .get("long_name")
            .cloned()
            .or_else(|| attributes.get("standard_name").cloned())
            .or_else(|| attributes.get("description").cloned())
            .or_else(|| attributes.get("title").cloned());

        let time_coverage_start = attributes
            .get("time_coverage_start")
            .cloned()
            .or_else(|| global_attrs.get("time_coverage_start").cloned());
        let time_coverage_end = attributes
            .get("time_coverage_end")
            .cloned()
            .or_else(|| global_attrs.get("time_coverage_end").cloned());
        let temporal_resolution = attributes
            .get("temporal_resolution")
            .cloned()
            .or_else(|| global_attrs.get("temporal_resolution").cloned());

        let file_size = crate::utils::units::calculate_variable_size_bytes(&shape, &data_type);

        variables.push(VariableInfo {
            name,
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

    for sub in group.groups() {
        let sub_name = sub.name();
        let new_prefix = if prefix.is_empty() {
            sub_name
        } else {
            format!("{prefix}/{sub_name}")
        };
        collect_group_variables(&sub, &new_prefix, global_attrs, variables);
    }
}

pub fn inspect_netcdf_file(file_path: &str) -> Result<DatasetMetadata, BlockStoreError> {
    let file = netcdf::open(file_path)
        .map_err(|e| format!("Failed to open NetCDF file '{file_path}': {e}"))?;

    let global_attrs = extract_global_attributes(&file);
    let mut variables = Vec::new();

    for var in file.variables() {
        let name = var.name();
        let vartype = var.vartype();
        let data_type = var_type_to_string(&vartype).to_string();

        let dims = var.dimensions();
        let shape: Vec<u64> = dims.iter().map(|d| d.len() as u64).collect();
        let dimension_names: Vec<String> = dims.iter().map(|d| d.name()).collect();
        let chunk_shape = shape.clone();

        let attributes = extract_variable_attributes(&var);

        let units = attributes.get("units").cloned();
        let long_name = attributes
            .get("long_name")
            .cloned()
            .or_else(|| attributes.get("standard_name").cloned())
            .or_else(|| attributes.get("description").cloned())
            .or_else(|| attributes.get("title").cloned());

        let time_coverage_start = attributes
            .get("time_coverage_start")
            .cloned()
            .or_else(|| global_attrs.get("time_coverage_start").cloned());
        let time_coverage_end = attributes
            .get("time_coverage_end")
            .cloned()
            .or_else(|| global_attrs.get("time_coverage_end").cloned());
        let temporal_resolution = attributes
            .get("temporal_resolution")
            .cloned()
            .or_else(|| global_attrs.get("temporal_resolution").cloned());

        let file_size = crate::utils::units::calculate_variable_size_bytes(&shape, &data_type);

        variables.push(VariableInfo {
            name,
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

    if let Ok(subgroups) = file.groups() {
        for sub in subgroups {
            let sub_name = sub.name();
            collect_group_variables(&sub, &sub_name, &global_attrs, &mut variables);
        }
    }

    let dimension_coordinates = extract_dimension_coordinates(&file, &variables);

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

    Ok(DatasetMetadata {
        name: dataset_name,
        store_type: "NetCDF".to_string(),
        variables,
        dimension_coordinates,
    })
}
