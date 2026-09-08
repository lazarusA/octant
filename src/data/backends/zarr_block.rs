//! Generic, N-dimensional fetch that returns an `OctantBlock` from a Zarr array.

use std::collections::HashMap;

use zarrs::array::ArraySubset;
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::storage::ReadableWritableListableStorage;

use super::generic_zarr::ZarrArrayHandle;
use super::zarr_slice::retrieve_array_subset_as_f32;
use crate::data::block_store::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::coordinates::get_cached_coord_bounds_scoped;
use crate::utils::grid::check_and_orient_block_grid;

/// Fetches an arbitrary-rank hyperslab described by `request` and returns it
/// as a resident `OctantBlock`.
pub fn fetch_block(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    fetch_block_with_progress(store, store_url, request, None)
}

/// Fetches an arbitrary-rank hyperslab from an already-opened `ZarrArrayHandle` through a chunk cache.
pub fn fetch_block_from_cached_array(
    array: &ZarrArrayHandle,
    cache: &ChunkCacheDecodedLruSizeLimit,
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    mut on_progress: crate::data::block_store::ProgressCallback,
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

    let attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    let group_path = request
        .variable
        .rfind('/')
        .map(|idx| &request.variable[..idx]);

    let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
    let total_dims = dim_names.len();
    for (i, name) in dim_names.iter().enumerate() {
        if let Some((first, last)) = get_cached_coord_bounds_scoped(
            store.clone(),
            store_url,
            name,
            group_path,
            &[],
            i,
            total_dims,
        ) {
            coordinates.insert(name.clone(), vec![first, last]);
        }
    }

    // Fallback: If dim_names contain generic "dim_i" names, query spatial coordinate bounds for lat and lon
    if coordinates.is_empty() || dim_names.iter().any(|d| d.starts_with("dim_")) {
        for candidate in &["lat", "latitude", "y", "lon", "longitude", "x"] {
            if let Some((first, last)) = get_cached_coord_bounds_scoped(
                store.clone(),
                store_url,
                candidate,
                group_path,
                &[],
                usize::MAX,
                total_dims,
            ) {
                coordinates.insert((*candidate).to_string(), vec![first, last]);
            }
        }
    }

    let mut curvilinear_coordinates: HashMap<String, crate::data::CurvilinearCoord2D> = HashMap::new();
    let rank = block_shape.len();
    if rank >= 2 {
        let y_dim_idx = rank - 2;
        let x_dim_idx = rank - 1;
        let y_full_len = array.shape().get(y_dim_idx).copied().unwrap_or(1) as usize;
        let x_full_len = array.shape().get(x_dim_idx).copied().unwrap_or(1) as usize;

        let candidate_names = [
            "nav_lon",
            "nav_lat",
            "lon",
            "lat",
            "longitude",
            "latitude",
            "x_lon",
            "y_lat",
        ];

        for cand in candidate_names {
            let group_candidates = if group_path.is_some() {
                vec![
                    format!("{}/{}", group_path.unwrap_or(""), cand),
                    format!("/{}", cand),
                    cand.to_string(),
                ]
            } else {
                vec![format!("/{}", cand), cand.to_string()]
            };

            for path in group_candidates {
                let clean_path = path.trim_start_matches('/');
                let Some(coord_array) = crate::utils::metadata::open_or_instantiate_array_normalized(
                    store.clone(),
                    clean_path,
                )
                .ok()
                else {
                    continue;
                };

                let coord_shape = coord_array.shape();
                let ok_rank = coord_shape.len() == 2;
                let coord_h = coord_shape[0] as usize;
                let coord_w = coord_shape[1] as usize;
                let matches_shape = ok_rank
                    && ((coord_h == y_full_len && coord_w == x_full_len)
                        || (coord_h == x_full_len && coord_w == y_full_len));
                if !matches_shape {
                    continue;
                }

                let y_start = origin.get(y_dim_idx).copied().unwrap_or(0).min(y_full_len.saturating_sub(1));
                let y_count = block_shape.get(y_dim_idx).copied().unwrap_or(1).min(y_full_len - y_start).max(1);
                let x_start = origin.get(x_dim_idx).copied().unwrap_or(0).min(x_full_len.saturating_sub(1));
                let x_count = block_shape.get(x_dim_idx).copied().unwrap_or(1).min(x_full_len - x_start).max(1);

                let coord_subset = ArraySubset::new_with_ranges(&[
                    y_start as u64..(y_start + y_count) as u64,
                    x_start as u64..(x_start + x_count) as u64,
                ]);

                if let Ok(vals) = retrieve_array_subset_as_f32(&coord_array, None, &coord_subset)
                    && !vals.is_empty()
                {
                    curvilinear_coordinates.insert(
                        cand.to_string(),
                        crate::data::CurvilinearCoord2D {
                            values: vals.into(),
                            width: x_count,
                            height: y_count,
                        },
                    );
                    break;
                }
            }

            if curvilinear_coordinates.contains_key(cand) {
                break;
            }
        }
    }

    let raw_values = check_and_orient_block_grid(
        raw_values,
        &mut block_shape,
        &mut dim_names,
        &mut origin,
        array.attributes(),
        &coordinates,
    );

    Ok(OctantBlock::new(
        request.variable.clone(),
        block_shape,
        dim_names,
        origin,
        raw_values,
        coordinates,
        attributes,
    )
    .with_curvilinear_coordinates(curvilinear_coordinates))
}

/// Fetches an arbitrary-rank hyperslab described by `request` with progress reporting.
pub fn fetch_block_with_progress(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    on_progress: crate::data::block_store::ProgressCallback,
) -> Result<OctantBlock, BlockStoreError> {
    let dummy_store =
        super::generic_zarr::GenericZarrBlockStore::new(store.clone(), store_url, "zarr", "Zarr");
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
