//! Zarr block retrieval, slicing, and coordinate binding routines.

use std::collections::HashMap;
use zarrs::array::ArraySubset;
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::storage::ReadableWritableListableStorage;

use super::block_coords::{BlockWindow, block_coordinates};
use super::generic::{GenericZarrBlockStore, ZarrArrayHandle};
use super::slice::retrieve_array_subset_as_f32;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::grid::check_and_orient_block_grid;
use crate::utils::grid_flips::OrientHints;
use crate::utils::metadata::attr_string;

/// The clamped `(origin, shape)` window of every dimension selected by `request`.
fn selection_window(
    request: &SliceRequest,
    shape: &[u64],
) -> Result<(Vec<usize>, Vec<usize>), BlockStoreError> {
    if request.selections.len() != shape.len() {
        return Err(format!(
            "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
            request.selections.len(),
            request.variable,
            shape.len()
        )
        .into());
    }
    let mut origin = Vec::with_capacity(shape.len());
    let mut block_shape = Vec::with_capacity(shape.len());
    for (sel, &dim_len) in request.selections.iter().zip(shape) {
        let dim_len = usize::try_from(dim_len).unwrap_or(usize::MAX);
        let (start, end) = match sel {
            DimensionSelection::Index(idx) => (*idx, idx.saturating_add(1)),
            DimensionSelection::Range { start, end } => (*start, *end),
        };
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start.saturating_add(1)).min(dim_len);
        origin.push(start);
        block_shape.push(end - start);
    }
    Ok((origin, block_shape))
}

/// The array's attributes as text, plus those of its ancestor groups (e.g. DGGS
/// conventions) when the array has no `dggs` attribute of its own.
fn inherited_attributes(
    array: &ZarrArrayHandle,
    store: &ReadableWritableListableStorage,
    variable: &str,
) -> HashMap<String, String> {
    let mut attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), attr_string(v)))
        .collect();
    if attributes.contains_key("dggs") {
        return attributes;
    }
    for ancestor in crate::utils::path::ancestor_paths(variable) {
        let p = if ancestor.is_empty() {
            "/".to_string()
        } else {
            format!("/{ancestor}")
        };
        if let Ok(grp) = zarrs::group::Group::open(store.clone(), &p) {
            for (k, v) in grp.attributes() {
                attributes
                    .entry(k.clone())
                    .or_insert_with(|| attr_string(v));
            }
            if attributes.contains_key("dggs") {
                break;
            }
        }
    }
    attributes
}

/// The values of the window `origin..origin + shape` of `array`, through `cache`.
fn read_window(
    array: &ZarrArrayHandle,
    cache: &ChunkCacheDecodedLruSizeLimit,
    variable: &str,
    (origin, shape): (&[usize], &[usize]),
) -> Result<Vec<f32>, BlockStoreError> {
    let ranges: Vec<_> = origin
        .iter()
        .zip(shape)
        .map(|(&start, &len)| start as u64..(start + len) as u64)
        .collect();
    let subset = ArraySubset::new_with_ranges(&ranges);
    log::debug!(
        "[ZarrBlock] Fetching '{variable}' subset {:?} (elements = {})",
        subset.to_ranges(),
        subset.num_elements(),
    );
    retrieve_array_subset_as_f32(array, Some(cache), &subset).map_err(|e| {
        log::error!("[ZarrBlock] Failed to retrieve array subset for '{variable}': {e:?}");
        BlockStoreError::from(e.to_string())
    })
}

/// Fetches an arbitrary-rank hyperslab from an already-opened `ZarrArrayHandle` through a chunk cache.
pub fn fetch_block_from_cached_array(
    array: &ZarrArrayHandle,
    cache: &ChunkCacheDecodedLruSizeLimit,
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    mut on_progress: ProgressCallback,
) -> Result<OctantBlock, BlockStoreError> {
    let full_shape = array.shape();
    let (mut origin, mut block_shape) = selection_window(request, full_shape)?;
    let raw_values = read_window(array, cache, &request.variable, (&origin, &block_shape))?;
    if let Some(ref mut cb) = on_progress {
        cb((raw_values.len() * std::mem::size_of::<f32>()) as u64);
    }

    let attributes = inherited_attributes(array, &store, &request.variable);
    let mut dim_names = crate::utils::resolve_array_dimension_names(array);
    let window = BlockWindow {
        dim_names: &dim_names,
        full_shape,
        origin: &origin,
        block_shape: &block_shape,
        selections: &request.selections,
    };
    let mut coords = block_coordinates(&store, store_url, &request.variable, &window);
    let hints = OrientHints {
        attributes: array.attributes(),
        lat_extent: coords.lat_extent,
        lon_extent: coords.lon_extent,
    };
    let (raw_values, flipped_dims) = check_and_orient_block_grid(
        raw_values,
        &mut block_shape,
        &mut dim_names,
        &mut origin,
        hints,
        &mut coords.coordinates,
    );
    let mut block = OctantBlock::new(
        request.variable.clone(),
        block_shape,
        dim_names,
        origin,
        raw_values,
        coords.coordinates,
        attributes,
    );
    block.flipped_dims = flipped_dims;
    Ok(block)
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
