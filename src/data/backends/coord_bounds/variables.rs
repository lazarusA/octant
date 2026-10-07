//! Dimension coordinates of a store's variables, aware of the groups they live in.

use std::collections::HashMap;
use zarrs::storage::ReadableWritableListableStorage;

use super::cache::get_cached_coord_values_scoped;
use crate::data::{CoordValues, VariableInfo};

/// Fetches all dimension coordinate values aware of variable group paths and hierarchy.
///
/// Every variable in a group gets `{variable}/{dim}` keys (lowercase, the form
/// `DatasetMetadata::get_dim_coords` looks up) holding its own group's coordinate; the
/// unscoped `{dim}` key holds the first one found, in variable order.
pub fn fetch_all_dimension_coordinates_for_variables(
    store: ReadableWritableListableStorage,
    variables: &[VariableInfo],
    store_url_hint: Option<&str>,
) -> HashMap<String, CoordValues> {
    let mut coords_map = HashMap::new();
    let url_hint = store_url_hint.unwrap_or("local");
    let group_prefixes = group_prefixes(variables);
    for var in variables {
        add_omero_labels(&mut coords_map, var);
    }
    for var in variables {
        let total_dims = var.dimension_names.len();
        for (i, name) in var.dimension_names.iter().enumerate() {
            let keys = DimKeys::new(var, name);
            let resolved = match &keys.scoped {
                Some(scoped) => coords_map.contains_key(scoped),
                None => coords_map.contains_key(&keys.clean),
            };
            if resolved {
                continue;
            }
            let group = keys.scoped.as_ref().and(var.group_path());
            let read = |scope| {
                get_cached_coord_values_scoped(
                    store.clone(),
                    url_hint,
                    name,
                    scope,
                    &group_prefixes,
                    i,
                    total_dims,
                )
            };
            // Grouped variables fall back to the root when their group has no coordinate.
            if let Some(values) = read(group).or_else(|| group.and_then(|_| read(None))) {
                keys.insert(&mut coords_map, values);
            }
        }
    }
    coords_map
}

/// Every group path and ancestor group path of `variables`, in first-seen order.
fn group_prefixes(variables: &[VariableInfo]) -> Vec<String> {
    let mut prefixes: Vec<String> = Vec::new();
    for gp in variables.iter().filter_map(VariableInfo::group_path) {
        let mut curr = String::new();
        for seg in gp.split('/') {
            if !curr.is_empty() {
                curr.push('/');
            }
            curr.push_str(seg);
            if !prefixes.contains(&curr) {
                prefixes.push(curr.clone());
            }
        }
    }
    prefixes
}

/// OME-Zarr channel names (`omero_channels`, comma separated) as labels of `var`'s
/// channel dimensions; empty names are skipped.
fn add_omero_labels(coords_map: &mut HashMap<String, CoordValues>, var: &VariableInfo) {
    let Some(ch_str) = var.attributes.get("omero_channels") else {
        return;
    };
    let labels: Vec<String> = ch_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let Some(labels) = CoordValues::from_labels(labels) else {
        return;
    };
    for name in &var.dimension_names {
        if crate::data::coordinates::naming::is_channel_dim_name(name) {
            DimKeys::new(var, name).insert(coords_map, labels.clone());
        }
    }
}

/// The keys a variable's dimension coordinate is stored under.
struct DimKeys<'a> {
    name: &'a str,
    clean: String,
    /// `{variable}/{dim}` for variables in a group.
    scoped: Option<String>,
}

impl<'a> DimKeys<'a> {
    fn new(var: &VariableInfo, name: &'a str) -> Self {
        let clean = name.trim().to_lowercase();
        let scoped = var
            .group_path()
            .map(|_| format!("{}/{clean}", var.name.trim().to_lowercase()));
        Self {
            name,
            clean,
            scoped,
        }
    }

    /// Stores `values` under the scoped key, and under the unscoped ones when still free.
    fn insert(self, coords_map: &mut HashMap<String, CoordValues>, values: CoordValues) {
        if self.clean != self.name {
            coords_map
                .entry(self.name.to_string())
                .or_insert_with(|| values.clone());
        }
        if let Some(scoped) = self.scoped {
            coords_map
                .entry(self.clean)
                .or_insert_with(|| values.clone());
            coords_map.insert(scoped, values);
        } else {
            coords_map.entry(self.clean).or_insert(values);
        }
    }
}
