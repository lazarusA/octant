//! Thread-safe global caching for coordinate bounds and boundary values.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use zarrs::storage::ReadableWritableListableStorage;

use super::extract::read_coord_values_scoped;
use crate::data::CoordValues;

/// Cache key: store URL (see [`cache_url`]), group scope (empty at the root), lowercase
/// dimension name.
type CacheKey = (String, String, String);

#[allow(clippy::type_complexity)]
static COORD_VALUES_CACHE: OnceLock<RwLock<HashMap<CacheKey, Option<CoordValues>>>> =
    OnceLock::new();

/// Drops the cached coordinates of the store at `store_url` (all stores for `None`), so a
/// closed dataset's coordinates do not stay in memory for the rest of the session.
pub fn evict_coord_values(store_url: Option<&str>) {
    let Some(lock) = COORD_VALUES_CACHE.get() else {
        return;
    };
    let mut cache = lock.write().unwrap_or_else(|p| p.into_inner());
    match store_url {
        Some(url) => {
            let url = cache_url(url);
            cache.retain(|(key_url, _, _), _| *key_url != url);
        }
        None => cache.clear(),
    }
}

/// The store part of cache keys: without surrounding whitespace, trailing slashes or the
/// `icechunk+` scheme prefix. Paths stay case-sensitive.
fn cache_url(store_url: &str) -> String {
    store_url
        .trim()
        .trim_start_matches("icechunk+")
        .trim_end_matches('/')
        .to_string()
}

/// A group scope without surrounding slashes or whitespace; `None` at the root.
fn clean_scope(group_scope: Option<&str>) -> Option<&str> {
    group_scope
        .map(|g| g.trim().trim_matches('/'))
        .filter(|g| !g.is_empty())
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
    let scope = clean_scope(group_scope);
    let key = (
        cache_url(store_url),
        scope.unwrap_or_default().to_string(),
        dim_name.trim().to_lowercase(),
    );
    {
        let cache = cache_lock.read().unwrap_or_else(|p| p.into_inner());
        if let Some(values) = cache.get(&key) {
            return values.clone();
        }
        // Without a scope, a coordinate already read for any group of the store will do; a
        // scoped lookup reads its own group's coordinate first.
        if scope.is_none()
            && let Some(values) = cache
                .iter()
                .find(|((url, _, dim), v)| *url == key.0 && *dim == key.2 && v.is_some())
                .map(|(_, v)| v.clone())
        {
            return values;
        }
    }

    let values =
        read_coord_values_scoped(store, dim_name, scope, known_groups, dim_idx, total_dims);
    let mut cache = cache_lock.write().unwrap_or_else(|p| p.into_inner());
    cache.insert(key, values.clone());
    values
}
