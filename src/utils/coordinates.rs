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

    for name in dim_names {
        let clean = name.trim().to_lowercase();
        if coords_map.contains_key(&clean) {
            continue;
        }

        if let Some((first, last)) = get_cached_coord_bounds(store.clone(), url_hint, &clean) {
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
    let cache_lock = COORD_BOUNDS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let key = format!("{}:{}", store_url, dim_name.trim().to_lowercase());

    if let Ok(cache) = cache_lock.read()
        && let Some(bounds) = cache.get(&key)
    {
        return *bounds;
    }

    let bounds = read_coord_bounds(store, dim_name);
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
    let clean = dim_name.trim().to_lowercase();
    let array_path = format!("/{}", clean);
    let array = Array::open(store.clone(), &array_path)
        .or_else(|_| Array::open(store.clone(), &clean))
        .ok()?;

    let len = array.shape().first().copied().unwrap_or(0) as usize;
    if len < 2 {
        return None;
    }

    let subset_start = ArraySubset::new_with_ranges(&[0..1]);
    let subset_end = ArraySubset::new_with_ranges(&[(len as u64 - 1)..len as u64]);

    let v_start = array
        .retrieve_array_subset::<Vec<f64>>(&subset_start)
        .ok()
        .and_then(|v| v.first().copied())
        .or_else(|| {
            array
                .retrieve_array_subset::<Vec<f32>>(&subset_start)
                .ok()
                .and_then(|v| v.first().map(|&x| x as f64))
        })?;

    let v_end = array
        .retrieve_array_subset::<Vec<f64>>(&subset_end)
        .ok()
        .and_then(|v| v.first().copied())
        .or_else(|| {
            array
                .retrieve_array_subset::<Vec<f32>>(&subset_end)
                .ok()
                .and_then(|v| v.first().map(|&x| x as f64))
        })?;

    Some((v_start, v_end))
}
