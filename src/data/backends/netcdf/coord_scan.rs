//! Dimension coordinates of a NetCDF file, group by group: numeric coordinate variables,
//! and text labels from variables named after their dimension or listed in a
//! `coordinates` attribute. Variables in groups resolve their dimensions from their own
//! group up to the root.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::coord_read::{
    group_ancestors, group_of, read_labels, read_number_coordinate, text_dimension,
};
use crate::data::coordinates::naming::{is_spatial_x_name, is_spatial_y_name, is_spatial_z_name};
use crate::data::metadata::{CoordValues, VariableInfo};

/// Coordinates found in one group, keyed by name and lowercase name.
type GroupCoords = HashMap<String, CoordValues>;

/// Every dimension coordinate of `file`. Root coordinates are keyed by name; each variable
/// also gets `{variable}/{dimension}` keys for the coordinates its group chain provides,
/// and grouped coordinates fill unscoped keys still free, in group path order.
pub fn extract_dimension_coordinates(
    file: &netcdf::File,
    variables: &[VariableInfo],
) -> HashMap<String, CoordValues> {
    let aux = auxiliary_names(variables);
    // Sorted by path, so unscoped keys shared by several groups resolve the same way every run.
    let mut groups: BTreeMap<String, GroupCoords> = BTreeMap::new();
    groups.insert(String::new(), scan_variables(file.variables(), &aux));
    if let Ok(subgroups) = file.groups() {
        for group in subgroups {
            let name = group.name();
            scan_group(&group, &name, &aux, &mut groups);
        }
    }

    let mut coords = groups.get("").cloned().unwrap_or_default();
    for var in variables {
        for dim in &var.dimension_names {
            if let Some(values) = resolve(&groups, group_of(&var.name), dim) {
                let key = format!("{}/{}", var.name.to_lowercase(), dim.trim().to_lowercase());
                coords.insert(key, values.clone());
            }
        }
    }
    for (path, group) in &groups {
        if !path.is_empty() {
            for (key, values) in group {
                coords.entry(key.clone()).or_insert_with(|| values.clone());
            }
        }
    }
    coords
}

fn scan_group(
    group: &netcdf::Group<'_>,
    path: &str,
    aux: &HashSet<String>,
    groups: &mut BTreeMap<String, GroupCoords>,
) {
    groups.insert(path.to_string(), scan_variables(group.variables(), aux));
    for sub in group.groups() {
        let sub_path = format!("{path}/{}", sub.name());
        scan_group(&sub, &sub_path, aux, groups);
    }
}

/// The coordinates among one group's variables. Labels replace a numeric coordinate only
/// when it is a plain `0..n-1` index, as xarray writes for dimensions it has no values for.
fn scan_variables<'f>(
    vars: impl Iterator<Item = netcdf::Variable<'f>>,
    aux: &HashSet<String>,
) -> GroupCoords {
    let mut coords = GroupCoords::new();
    let mut labels = Vec::new();
    for var in vars {
        let name = var.name();
        // Only text variables that label a dimension are read.
        if let Some(dim) = text_dimension(&var) {
            if (name == dim || aux.contains(&name))
                && let Some(values) = read_labels(&var)
            {
                labels.push((dim, values));
            }
            continue;
        }
        let dims = var.dimensions();
        if let [dim] = dims
            && dim.len() > 0
            && is_coordinate_name(&name, &dim.name())
            && let Some(values) = read_number_coordinate(&var)
        {
            insert(&mut coords, &name, values);
        }
    }
    for (dim, values) in labels {
        let keeps_numbers = coords.get(&dim).is_some_and(|c| !is_index(c));
        if !keeps_numbers && let Some(values) = CoordValues::from_labels(values) {
            insert(&mut coords, &dim, values);
        }
    }
    coords
}

/// The coordinate of `dim` for a variable in `group`, searched from it up to the root.
fn resolve<'a>(
    groups: &'a BTreeMap<String, GroupCoords>,
    group: &str,
    dim: &str,
) -> Option<&'a CoordValues> {
    let clean = dim.trim().to_lowercase();
    group_ancestors(group).find_map(|path| {
        let coords = groups.get(path)?;
        coords.get(dim).or_else(|| coords.get(&clean))
    })
}

/// Variables listed in any `coordinates` attribute (CF auxiliary coordinates).
fn auxiliary_names(variables: &[VariableInfo]) -> HashSet<String> {
    variables
        .iter()
        .filter_map(|v| v.attributes.get("coordinates"))
        .flat_map(|list| list.split_whitespace())
        .map(str::to_string)
        .collect()
}

/// A 1D variable is a coordinate when named after its dimension or after a spatial,
/// time or vertical axis.
fn is_coordinate_name(name: &str, dim: &str) -> bool {
    let clean = name.trim().to_lowercase();
    dim == name
        || is_spatial_x_name(&clean)
        || is_spatial_y_name(&clean)
        || is_spatial_z_name(&clean)
        || matches!(clean.as_str(), "time" | "depth" | "lev" | "level")
}

/// Whether `values` are just the indices `0, 1, 2, ...`.
fn is_index(values: &CoordValues) -> bool {
    matches!(values, CoordValues::Regular { start, step, .. } if *start == 0.0 && *step == 1.0)
}

fn insert(coords: &mut GroupCoords, name: &str, values: CoordValues) {
    let clean = name.trim().to_lowercase();
    if clean != name {
        coords.insert(clean, values.clone());
    }
    coords.insert(name.to_string(), values);
}
