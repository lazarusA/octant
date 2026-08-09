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
        "{}:{}:{}",
        store_url,
        dim_name.trim().to_lowercase(),
        dim_idx
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
    dim_idx: usize,
    total_dims: usize,
) -> Option<(f64, f64)> {
    let clean = dim_name.trim().to_lowercase();
    let mut candidates = vec![format!("/{}", clean), clean.clone()];

    let is_lat = clean.contains("lat")
        || clean == "y"
        || (total_dims >= 2 && dim_idx == total_dims.saturating_sub(2));

    let is_lon = clean.contains("lon")
        || clean == "x"
        || (total_dims >= 2 && dim_idx == total_dims.saturating_sub(1));

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
