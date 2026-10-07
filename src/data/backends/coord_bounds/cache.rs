//! Thread-safe global caching for coordinate bounds and boundary values.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use zarrs::storage::ReadableWritableListableStorage;

use super::extract::read_coord_values_scoped;
use crate::data::CoordValues;

#[allow(clippy::type_complexity)]
static COORD_VALUES_CACHE: OnceLock<RwLock<HashMap<String, Option<CoordValues>>>> = OnceLock::new();

/// Drops the cached coordinates of the store at `store_url` (all stores for `None`), so a
/// closed dataset's coordinates do not stay in memory for the rest of the session.
pub fn evict_coord_values(store_url: Option<&str>) {
    let Some(lock) = COORD_VALUES_CACHE.get() else {
        return;
    };
    let mut cache = lock.write().unwrap_or_else(|p| p.into_inner());
    match store_url {
        Some(url) => {
            let prefix = format!("{}:", cache_url(url));
            cache.retain(|key, _| !key.starts_with(&prefix));
        }
        None => cache.clear(),
    }
}

/// The store part of cache keys: case- and trailing-slash-insensitive.
fn cache_url(store_url: &str) -> String {
    store_url.trim().trim_end_matches('/').to_lowercase()
}

/// The first and last coordinate values, in storage order; `None` for labels.
#[inline]
pub(crate) fn parse_bounds_from_values(values: &CoordValues) -> Option<(f64, f64)> {
    Some((values.first_number()?, values.last_number()?))
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
    let values = get_cached_coord_values_scoped(
        store,
        store_url,
        dim_name,
        group_scope,
        known_groups,
        dim_idx,
        total_dims,
    )?;
    parse_bounds_from_values(&values)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn get_cached_coord_values_with_rank(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
    dim_idx: usize,
    total_dims: usize,
) -> Option<CoordValues> {
    get_cached_coord_values_scoped(store, store_url, dim_name, None, &[], dim_idx, total_dims)
}

#[allow(clippy::single_range_in_vec_init)]
pub fn get_cached_coord_values_scoped(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    dim_idx: usize,
    total_dims: usize,
) -> Option<CoordValues> {
    let cache_lock = COORD_VALUES_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let clean_url = cache_url(store_url);
    let clean_dim = dim_name.trim().to_lowercase();
    let exact_key = format!("{}:{}:{}", clean_url, group_scope.unwrap_or(""), clean_dim);

    {
        let cache = cache_lock.read().unwrap_or_else(|p| p.into_inner());
        if let Some(values) = cache.get(&exact_key) {
            return values.clone();
        }
        let root_key = format!("{}::{}", clean_url, clean_dim);
        if let Some(values) = cache.get(&root_key) {
            return values.clone();
        }
        let url_prefix = format!("{}:", clean_url);
        let dim_suffix = format!(":{}", clean_dim);
        for (k, v) in cache.iter() {
            if k.starts_with(&url_prefix)
                && k.ends_with(&dim_suffix)
                && let Some(values) = v
            {
                return Some(values.clone());
            }
        }
    }

    let values = read_coord_values_scoped(
        store,
        dim_name,
        group_scope,
        known_groups,
        dim_idx,
        total_dims,
    );
    let mut cache = cache_lock.write().unwrap_or_else(|p| p.into_inner());
    cache.insert(exact_key, values.clone());
    values
}
