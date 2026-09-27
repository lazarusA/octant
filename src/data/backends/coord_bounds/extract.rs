//! Coordinate range extraction and subset retrieval from storage arrays.

use std::collections::HashMap;
use zarrs::array::ArraySubset;
use zarrs::storage::ReadableWritableListableStorage;

use super::cache::{
    get_cached_coord_values_scoped, get_cached_coord_values_with_rank, parse_bounds_from_values,
};
use super::discover::discover_coord_array;
use crate::data::VariableInfo;

/// Fetches all dimension coordinate values for the specified dimension names across the store.
pub fn fetch_all_dimension_coordinates(
    store: ReadableWritableListableStorage,
    dim_names: &[String],
    store_url_hint: Option<&str>,
) -> HashMap<String, Vec<String>> {
    let mut coords_map = HashMap::new();
    let url_hint = store_url_hint.unwrap_or("local");
    let total_dims = dim_names.len();

    for (i, name) in dim_names.iter().enumerate() {
        let clean = name.trim().to_lowercase();
        if coords_map.contains_key(&clean) {
            continue;
        }

        if let Some(values) =
            get_cached_coord_values_with_rank(store.clone(), url_hint, name, i, total_dims)
        {
            coords_map.insert(clean.clone(), values.clone());
            if clean != *name {
                coords_map.insert(name.clone(), values);
            }
        }
    }

    coords_map
}

/// Fetches all dimension coordinate values aware of variable group paths and hierarchy.
pub fn fetch_all_dimension_coordinates_for_variables(
    store: ReadableWritableListableStorage,
    variables: &[VariableInfo],
    store_url_hint: Option<&str>,
) -> HashMap<String, Vec<String>> {
    let mut coords_map = HashMap::new();
    let url_hint = store_url_hint.unwrap_or("local");

    // Collect all group prefixes from variables
    let mut group_prefixes = Vec::new();
    for var in variables {
        if let Some(gp) = var.group_path() {
            let mut curr = String::new();
            for seg in gp.split('/') {
                if !curr.is_empty() {
                    curr.push('/');
                }
                curr.push_str(seg);
                if !group_prefixes.contains(&curr) {
                    group_prefixes.push(curr.clone());
                }
            }
        }
        if let Some(ch_str) = var.attributes.get("omero_channels") {
            let labels: Vec<String> = ch_str.split(',').map(|s| s.trim().to_string()).collect();
            for name in &var.dimension_names {
                if crate::data::coordinates::naming::is_channel_dim_name(name) {
                    let clean = name.trim().to_lowercase();
                    coords_map.insert(clean.clone(), labels.clone());
                    if clean != *name {
                        coords_map.insert(name.clone(), labels.clone());
                    }
                }
            }
        }
    }

    for var in variables {
        let group_path = var.group_path();
        let total_dims = var.dimension_names.len();
        for (i, name) in var.dimension_names.iter().enumerate() {
            let clean = name.trim().to_lowercase();
            if coords_map.contains_key(&clean) {
                continue;
            }

            if let Some(values) = get_cached_coord_values_scoped(
                store.clone(),
                url_hint,
                name,
                group_path,
                &group_prefixes,
                i,
                total_dims,
            ) {
                coords_map.insert(clean.clone(), values.clone());
                if clean != *name {
                    coords_map.insert(name.clone(), values.clone());
                }
                if let Some(gp) = group_path {
                    let scoped_key = format!("{}/{}", gp.trim().to_lowercase(), clean);
                    coords_map.insert(scoped_key, values);
                }
            }
        }
    }

    // Also fallback to root dim names if any remain unresolved
    for var in variables {
        for (i, name) in var.dimension_names.iter().enumerate() {
            let clean = name.trim().to_lowercase();
            if !coords_map.contains_key(&clean)
                && let Some(values) = get_cached_coord_values_scoped(
                    store.clone(),
                    url_hint,
                    name,
                    None,
                    &group_prefixes,
                    i,
                    var.dimension_names.len(),
                )
            {
                coords_map.insert(clean.clone(), values.clone());
                if clean != *name {
                    coords_map.insert(name.clone(), values);
                }
            }
        }
    }

    coords_map
}

#[allow(clippy::single_range_in_vec_init)]
pub fn read_coord_bounds(
    store: ReadableWritableListableStorage,
    dim_name: &str,
) -> Option<(f64, f64)> {
    read_coord_bounds_with_rank(store, dim_name, usize::MAX, 0)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn read_coord_bounds_with_rank(
    store: ReadableWritableListableStorage,
    dim_name: &str,
    dim_idx: usize,
    total_dims: usize,
) -> Option<(f64, f64)> {
    read_coord_bounds_scoped(store, dim_name, None, &[], dim_idx, total_dims)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn read_coord_bounds_scoped(
    store: ReadableWritableListableStorage,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    dim_idx: usize,
    total_dims: usize,
) -> Option<(f64, f64)> {
    let values = read_coord_values_scoped(
        store,
        dim_name,
        group_scope,
        known_groups,
        dim_idx,
        total_dims,
    )?;
    parse_bounds_from_values(&values)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn read_coord_values_scoped(
    store: ReadableWritableListableStorage,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    _dim_idx: usize,
    _total_dims: usize,
) -> Option<Vec<String>> {
    let clean = dim_name.trim().to_lowercase();
    let mut candidates = Vec::new();
    let mut add_candidate = |p: String| {
        let clean_key = p.trim_start_matches('/').to_string();
        if !clean_key.is_empty() && !candidates.contains(&clean_key) {
            candidates.push(clean_key);
        }
    };

    if let Some(scope) = group_scope {
        add_candidate(format!("{}/{}", scope, clean));
    }
    add_candidate(clean.clone());

    let is_lat = clean.contains("lat") || clean == "y";
    let is_lon = clean.contains("lon") || clean == "x";

    let aliases: &[&str] = if is_lat {
        &["lat", "latitude", "y", "coords/lat", "coordinates/latitude"]
    } else if is_lon {
        &[
            "lon",
            "longitude",
            "x",
            "coords/lon",
            "coordinates/longitude",
        ]
    } else {
        &[]
    };

    for alias in aliases {
        if let Some(scope) = group_scope {
            add_candidate(format!("{}/{}", scope, alias));
        }
        add_candidate(alias.to_string());
    }

    for group in known_groups {
        add_candidate(format!("{}/{}", group, clean));
        for alias in aliases {
            add_candidate(format!("{}/{}", group, alias));
        }
    }

    let array = discover_coord_array(store, &candidates, aliases)?;

    let len = array.shape().first().copied().unwrap_or(0) as usize;
    if len == 0 {
        return None;
    }
    if len == 1 {
        let subset_0 = ArraySubset::new_with_ranges(&[0..1]);
        let val =
            crate::data::backends::zarr::retrieve_array_subset_as_f32(&array, None, &subset_0)
                .ok()
                .and_then(|v| v.first().copied())?;
        return Some(vec![val.to_string()]);
    }

    // Retrieve boundary coordinates (first, last) for O(1) memory and instant resolution
    let subset_start = ArraySubset::new_with_ranges(&[0..1]);
    let subset_end = ArraySubset::new_with_ranges(&[(len as u64 - 1)..len as u64]);

    let v_start =
        crate::data::backends::zarr::retrieve_array_subset_as_f32(&array, None, &subset_start)
            .ok()
            .and_then(|v| v.first().map(|&x| x as f64))?;

    let v_end =
        crate::data::backends::zarr::retrieve_array_subset_as_f32(&array, None, &subset_end)
            .ok()
            .and_then(|v| v.first().map(|&x| x as f64))?;

    Some(vec![v_start.to_string(), v_end.to_string()])
}
