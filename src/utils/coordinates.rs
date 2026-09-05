use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorage;

#[allow(clippy::type_complexity)]
static COORD_BOUNDS_CACHE: OnceLock<RwLock<HashMap<String, Option<(f64, f64)>>>> = OnceLock::new();

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

        if let Some((first, last)) =
            get_cached_coord_bounds_with_rank(store.clone(), url_hint, name, i, total_dims)
        {
            coords_map.insert(clean, vec![first.to_string(), last.to_string()]);
        }
    }

    coords_map
}

/// Fetches all dimension coordinate values aware of variable group paths and hierarchy.
pub fn fetch_all_dimension_coordinates_for_variables(
    store: ReadableWritableListableStorage,
    variables: &[crate::data::VariableInfo],
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
    }

    for var in variables {
        let group_path = var.group_path();
        let total_dims = var.dimension_names.len();
        for (i, name) in var.dimension_names.iter().enumerate() {
            let clean = name.trim().to_lowercase();
            if coords_map.contains_key(&clean) {
                continue;
            }

            if let Some((first, last)) = get_cached_coord_bounds_scoped(
                store.clone(),
                url_hint,
                name,
                group_path,
                &group_prefixes,
                i,
                total_dims,
            ) {
                coords_map.insert(clean.clone(), vec![first.to_string(), last.to_string()]);
            }
        }
    }

    // Also fallback to generic dim names if any remain unresolved
    let all_dim_names: Vec<String> = variables
        .iter()
        .flat_map(|v| v.dimension_names.clone())
        .collect();
    for (i, name) in all_dim_names.iter().enumerate() {
        let clean = name.trim().to_lowercase();
        if !coords_map.contains_key(&clean)
            && let Some((first, last)) = get_cached_coord_bounds_with_rank(
                store.clone(),
                url_hint,
                name,
                i,
                all_dim_names.len(),
            )
        {
            coords_map.insert(clean, vec![first.to_string(), last.to_string()]);
        }
    }

    coords_map
}

#[allow(clippy::single_range_in_vec_init)]
pub fn get_cached_coord_bounds(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
) -> Option<(f64, f64)> {
    get_cached_coord_bounds_with_rank(store, store_url, dim_name, usize::MAX, 0)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn get_cached_coord_bounds_with_rank(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
    dim_idx: usize,
    total_dims: usize,
) -> Option<(f64, f64)> {
    get_cached_coord_bounds_scoped(store, store_url, dim_name, None, &[], dim_idx, total_dims)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn get_cached_coord_bounds_scoped(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    dim_idx: usize,
    total_dims: usize,
) -> Option<(f64, f64)> {
    let cache_lock = COORD_BOUNDS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let key = format!(
        "{}:{}:{}",
        store_url.trim().to_lowercase(),
        group_scope.unwrap_or(""),
        dim_name.trim().to_lowercase()
    );

    if let Ok(cache) = cache_lock.read()
        && let Some(bounds) = cache.get(&key)
    {
        return *bounds;
    }

    let bounds = read_coord_bounds_scoped(
        store,
        dim_name,
        group_scope,
        known_groups,
        dim_idx,
        total_dims,
    );
    if let Ok(mut cache) = cache_lock.write() {
        cache.insert(key, bounds);
    }
    bounds
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
    _dim_idx: usize,
    _total_dims: usize,
) -> Option<(f64, f64)> {
    let clean = dim_name.trim().to_lowercase();
    let mut candidates = Vec::new();

    // 1. Group-scoped candidate path (e.g. atmosphere/forecast/lat)
    if let Some(scope) = group_scope {
        candidates.push(format!("{}/{}", scope, clean));
        candidates.push(format!("/{}/{}", scope, clean));
    }

    // 2. Direct root candidate paths
    candidates.push(format!("/{}", clean));
    candidates.push(clean.clone());

    let is_lat = clean.contains("lat") || clean == "y";
    let is_lon = clean.contains("lon") || clean == "x";

    // 3. Heuristic spatial coordinate names
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
            candidates.push(format!("{}/{}", scope, alias));
            candidates.push(format!("/{}/{}", scope, alias));
        }
        candidates.push(format!("/{}", alias));
        candidates.push(alias.to_string());
    }

    // 4. Known group prefixes across the store (e.g. nested-only stores)
    for group in known_groups {
        candidates.push(format!("{}/{}", group, clean));
        candidates.push(format!("/{}/{}", group, clean));
        for alias in aliases {
            candidates.push(format!("{}/{}", group, alias));
            candidates.push(format!("/{}/{}", group, alias));
        }
    }

    let mut found_array = None;
    for path in &candidates {
        let clean_path = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{}", path)
        };
        let clean_key = path.trim_start_matches('/');

        if let Ok(arr) = Array::open(store.clone(), &clean_path) {
            found_array = Some(arr);
            break;
        }
        if let Ok(arr) = Array::open(store.clone(), clean_key) {
            found_array = Some(arr);
            break;
        }
        // Direct StoreKey checks for Zarr v3 and v2
        if let Ok(key) = zarrs::storage::StoreKey::new(format!("{}/zarr.json", clean_key))
            && let Ok(Some(bytes)) = store.get(&key)
            && let Ok(node_meta) = serde_json::from_slice::<zarrs::node::NodeMetadata>(&bytes)
            && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
                store.clone(),
                &clean_path,
                &node_meta,
            )
        {
            found_array = Some(arr);
            break;
        }
        if let Ok(key) = zarrs::storage::StoreKey::new(format!("{}/.zarray", clean_key))
            && let Ok(Some(bytes)) = store.get(&key)
            && let Ok(node_meta) = serde_json::from_slice::<zarrs::node::NodeMetadata>(&bytes)
            && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
                store.clone(),
                &clean_path,
                &node_meta,
            )
        {
            found_array = Some(arr);
            break;
        }
    }

    if found_array.is_none()
        && let Ok(group) = zarrs::group::Group::open(store.clone(), "/")
        && let Some(zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata {
            metadata,
            ..
        }) = group.consolidated_metadata()
    {
        for path in &candidates {
            let key = path.trim_start_matches('/');
            if let Some(node_meta) = metadata.get(key).or_else(|| metadata.get(path))
                && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
                    store.clone(),
                    path,
                    node_meta,
                )
            {
                found_array = Some(arr);
                break;
            }
        }
    }
    let array = found_array?;

    let len = array.shape().first().copied().unwrap_or(0) as usize;
    if len < 2 {
        return None;
    }

    let subset_start = ArraySubset::new_with_ranges(&[0..1]);
    let subset_end = ArraySubset::new_with_ranges(&[(len as u64 - 1)..len as u64]);

    let v_start = crate::data::backends::zarr_slice::retrieve_array_subset_as_f32(
        &array,
        None,
        &subset_start,
    )
    .ok()
    .and_then(|v| v.first().map(|&x| x as f64))?;

    let v_end =
        crate::data::backends::zarr_slice::retrieve_array_subset_as_f32(&array, None, &subset_end)
            .ok()
            .and_then(|v| v.first().map(|&x| x as f64))?;

    Some((v_start, v_end))
}

/// Checks if a dimension name matches Spatial X heuristics (longitude / X / column).
pub fn is_spatial_x_name(dim_name: &str) -> bool {
    let clean = dim_name.trim().to_lowercase();
    clean.contains("lon") || clean == "x" || clean.contains("col")
}

/// Checks if a dimension name matches Spatial Y heuristics (latitude / Y / row).
pub fn is_spatial_y_name(dim_name: &str) -> bool {
    let clean = dim_name.trim().to_lowercase();
    clean.contains("lat") || clean == "y" || clean.contains("row")
}

/// Checks if a dimension name matches Spatial Z heuristics (depth / level / height / alt / sigma / Z).
pub fn is_spatial_z_name(dim_name: &str) -> bool {
    let clean = dim_name.trim().to_lowercase();
    clean.contains("depth")
        || clean.contains("level")
        || clean.contains("lev")
        || clean.contains("height")
        || clean.contains("alt")
        || clean.contains("sigma")
        || clean == "z"
}

/// Formats a dimension name into a human-friendly axis title with standard units.
pub fn format_dimension_axis_title(dim_name: &str) -> String {
    let clean = dim_name.trim().to_lowercase();
    if clean.is_empty() {
        return "Index".to_string();
    }
    if clean.contains('[') || clean.contains('(') {
        return dim_name.to_string();
    }

    if clean.contains("lon") {
        format!("{dim_name} [°E]")
    } else if clean.contains("lat") {
        format!("{dim_name} [°N]")
    } else if clean.contains("depth") || clean.contains("height") || clean.contains("alt") {
        format!("{dim_name} [m]")
    } else if clean.contains("time") {
        dim_name.to_string()
    } else if clean == "x" || clean == "y" || clean == "z" {
        format!("{dim_name} Index")
    } else {
        dim_name.to_string()
    }
}
