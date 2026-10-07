//! Coordinate range extraction and subset retrieval from storage arrays.

use std::collections::HashMap;
use zarrs::storage::ReadableWritableListableStorage;

use super::cache::{get_cached_coord_values_with_rank, parse_bounds_from_values};
use super::discover::discover_coord_array;
use crate::data::CoordValues;

/// Fetches all dimension coordinate values for the specified dimension names across the store.
pub fn fetch_all_dimension_coordinates(
    store: ReadableWritableListableStorage,
    dim_names: &[String],
    store_url_hint: Option<&str>,
) -> HashMap<String, CoordValues> {
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
) -> Option<CoordValues> {
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
    crate::data::backends::zarr::read_coordinate(&array)
}
