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
    let cache_lock = COORD_BOUNDS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let key = format!(
        "{}:{}",
        store_url.trim().to_lowercase(),
        dim_name.trim().to_lowercase()
    );

    if let Ok(cache) = cache_lock.read()
        && let Some(bounds) = cache.get(&key)
    {
        return *bounds;
    }

    let bounds = read_coord_bounds_with_rank(store, dim_name, dim_idx, total_dims);
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
    _dim_idx: usize,
    _total_dims: usize,
) -> Option<(f64, f64)> {
    let clean = dim_name.trim().to_lowercase();
    let mut candidates = vec![format!("/{}", clean), clean.clone()];

    let is_lat = clean.contains("lat") || clean == "y";
    let is_lon = clean.contains("lon") || clean == "x";

    if is_lat {
        candidates.push("/lat".to_string());
        candidates.push("lat".to_string());
        candidates.push("/latitude".to_string());
        candidates.push("latitude".to_string());
        candidates.push("/y".to_string());
        candidates.push("y".to_string());
        candidates.push("/coords/lat".to_string());
        candidates.push("/coordinates/latitude".to_string());
    } else if is_lon {
        candidates.push("/lon".to_string());
        candidates.push("lon".to_string());
        candidates.push("/longitude".to_string());
        candidates.push("longitude".to_string());
        candidates.push("/x".to_string());
        candidates.push("x".to_string());
        candidates.push("/coords/lon".to_string());
        candidates.push("/coordinates/longitude".to_string());
    }

    let mut found_array = None;
    for path in &candidates {
        if let Ok(arr) = Array::open(store.clone(), path) {
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

    let v_start =
        crate::data::backends::zarr_slice::retrieve_array_subset_as_f32(&array, &subset_start)
            .ok()
            .and_then(|v| v.first().map(|&x| x as f64))?;

    let v_end =
        crate::data::backends::zarr_slice::retrieve_array_subset_as_f32(&array, &subset_end)
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
