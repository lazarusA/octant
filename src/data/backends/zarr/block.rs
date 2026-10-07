//! Zarr block retrieval, slicing, and coordinate binding routines.

use std::collections::HashMap;
use zarrs::array::ArraySubset;
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::storage::ReadableWritableListableStorage;

use super::generic::{GenericZarrBlockStore, ZarrArrayHandle};
use super::slice::retrieve_array_subset_as_f32;
use crate::data::CoordValues;
use crate::data::backends::coord_bounds::get_cached_coord_values_scoped;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::grid::check_and_orient_block_grid;

/// Fetches an arbitrary-rank hyperslab from an already-opened `ZarrArrayHandle` through a chunk cache.
pub fn fetch_block_from_cached_array(
    array: &ZarrArrayHandle,
    cache: &ChunkCacheDecodedLruSizeLimit,
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    mut on_progress: ProgressCallback,
) -> Result<OctantBlock, BlockStoreError> {
    let shape = array.shape();
    let rank = shape.len();

    if request.selections.len() != rank {
        return Err(format!(
            "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
            request.selections.len(),
            request.variable,
            rank
        )
        .into());
    }

    let mut dim_names = crate::utils::resolve_array_dimension_names(array);
    let mut ranges: Vec<std::ops::Range<u64>> = Vec::with_capacity(rank);

    let mut block_shape = Vec::with_capacity(rank);
    let mut origin = Vec::with_capacity(rank);

    for (i, sel) in request.selections.iter().enumerate() {
        let dim_len = shape[i] as usize;
        let (start, end) = match sel {
            DimensionSelection::Index(idx) => (*idx, idx.saturating_add(1)),
            DimensionSelection::Range { start, end } => (*start, *end),
        };
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start + 1).min(dim_len);

        ranges.push(start as u64..end as u64);
        block_shape.push(end - start);
        origin.push(start);
    }

    let subset = ArraySubset::new_with_ranges(&ranges);
    log::debug!(
        "[ZarrBlock] Fetching '{}' subset {:?} (elements = {}) from '{}'",
        request.variable,
        subset.to_ranges(),
        subset.num_elements(),
        store_url
    );

    let raw_values = match retrieve_array_subset_as_f32(array, Some(cache), &subset) {
        Ok(vals) => vals,
        Err(e) => {
            log::error!(
                "[ZarrBlock] Failed to retrieve array subset for '{}': {:?}",
                request.variable,
                e
            );
            return Err(e.to_string().into());
        }
    };
    let bytes_read = (raw_values.len() * std::mem::size_of::<f32>()) as u64;

    if let Some(ref mut cb) = on_progress {
        cb(bytes_read);
    }

    let mut attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    let group_path = request
        .variable
        .rfind('/')
        .map(|idx| &request.variable[..idx]);

    // Inherit ancestor group attributes (e.g. DGGS conventions) if not present on array
    if !attributes.contains_key("dggs") {
        for ancestor in crate::utils::path::ancestor_paths(&request.variable) {
            let p = if ancestor.is_empty() {
                "/".to_string()
            } else {
                format!("/{ancestor}")
            };
            if let Ok(grp) = zarrs::group::Group::open(store.clone(), &p) {
                for (k, v) in grp.attributes() {
                    attributes.entry(k.clone()).or_insert_with(|| v.to_string());
                }
                if attributes.contains_key("dggs") {
                    break;
                }
            }
        }
    }

    let full_shape = array.shape();
    let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
    let total_dims = dim_names.len();
    let coord_values = |name: &str, dim_idx: usize| {
        get_cached_coord_values_scoped(
            store.clone(),
            store_url,
            name,
            group_path,
            &[],
            dim_idx,
            total_dims,
        )
    };
    let window = |i: usize, coords: &CoordValues| {
        let full_len = full_shape.get(i).copied().unwrap_or(block_shape[i] as u64) as usize;
        let start = origin.get(i).copied().unwrap_or(0);
        window_coords(
            coords,
            full_len,
            start,
            block_shape.get(i).copied().unwrap_or(1),
        )
    };
    for (i, name) in dim_names.iter().enumerate() {
        if let Some(values) = coord_values(name, i).and_then(|c| window(i, &c)) {
            let clean = name.trim().to_lowercase();
            if clean != *name {
                coordinates.insert(clean, values.clone());
            }
            coordinates.insert(name.clone(), values);
        }
    }

    // Fallback: If dim_names contain generic "dim_i" names, query spatial coordinate bounds for lat and lon
    if coordinates.is_empty() || dim_names.iter().any(|d| d.starts_with("dim_")) {
        for candidate in &["lat", "latitude", "y", "lon", "longitude", "x"] {
            let Some(coords) = coord_values(candidate, usize::MAX) else {
                continue;
            };
            let is_x = crate::data::coordinates::naming::is_spatial_x_name(candidate);
            let dim_i = dim_names.iter().position(|d| {
                if is_x {
                    crate::data::coordinates::naming::is_spatial_x_name(d)
                } else {
                    crate::data::coordinates::naming::is_spatial_y_name(d)
                }
            });
            let values = match dim_i {
                Some(i) => window(i, &coords),
                None => coords
                    .first_number()
                    .zip(coords.last_number())
                    .map(|(first, last)| vec![first, last]),
            };
            if let Some(values) = values {
                coordinates.insert((*candidate).to_string(), values);
            }
        }
    }

    let raw_values = check_and_orient_block_grid(
        raw_values,
        &mut block_shape,
        &mut dim_names,
        &mut origin,
        array.attributes(),
        &mut coordinates,
    );

    Ok(OctantBlock::new(
        request.variable.clone(),
        block_shape,
        dim_names,
        origin,
        raw_values,
        coordinates,
        attributes,
    ))
}

/// Fetches an arbitrary-rank hyperslab described by `request` with progress reporting.
pub fn fetch_block_with_progress(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    on_progress: ProgressCallback,
) -> Result<OctantBlock, BlockStoreError> {
    let dummy_store = GenericZarrBlockStore::new(store.clone(), store_url, "zarr", "Zarr");
    let (cached_array, cache) = dummy_store.get_or_open_array(&request.variable)?;
    fetch_block_from_cached_array(
        &cached_array,
        &cache,
        store,
        store_url,
        request,
        on_progress,
    )
}

/// Coordinates of the block window `start..start + count` along a dimension of `full_len`:
/// every value of an uneven coordinate (so the plot grid places each row), else the
/// window's exact first and last value.
pub(crate) fn window_coords(
    coords: &CoordValues,
    full_len: usize,
    start: usize,
    count: usize,
) -> Option<Vec<f64>> {
    let end = start + count.max(1) - 1;
    if matches!(coords, CoordValues::Values(_)) && coords.matches(full_len) {
        return (start..=end).map(|i| coords.number(i)).collect();
    }
    Some(vec![
        coords.number_for(start, full_len)?,
        coords.number_for(end, full_len)?,
    ])
}
