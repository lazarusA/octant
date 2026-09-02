//! Generic, N-dimensional fetch that returns an `OctantBlock` from a Zarr array.

use std::collections::HashMap;

use zarrs::array::ArraySubset;
use zarrs::storage::ReadableWritableListableStorage;

use super::generic_zarr::ZarrArrayHandle;
use super::zarr_slice::retrieve_array_subset_as_f32;
use crate::data::block_store::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::coordinates::get_cached_coord_bounds_with_rank;
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

/// Fetches an arbitrary-rank hyperslab from an already-opened `ZarrArrayHandle`.
pub fn fetch_block_from_cached_array(
    array: &ZarrArrayHandle,
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    mut on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
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

    let mut dim_names: Vec<String> = array
        .dimension_names()
        .as_ref()
        .map(|names| {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| n.clone().unwrap_or_else(|| format!("dim_{i}")))
                .collect()
        })
        .or_else(|| {
            array.attributes().get("_ARRAY_DIMENSIONS").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .enumerate()
                        .map(|(i, s)| {
                            s.as_str()
                                .map(|str_v| str_v.to_string())
                                .unwrap_or_else(|| format!("dim_{i}"))
                        })
                        .collect()
                })
            })
        })
        .unwrap_or_else(|| crate::utils::default_dimension_names_for_rank(rank));

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
    let raw_values = retrieve_array_subset_as_f32(array, &subset).map_err(|e| e.to_string())?;
    let bytes_read = (raw_values.len() * std::mem::size_of::<f32>()) as u64;
    if let Some(ref mut cb) = on_progress {
        cb(bytes_read);
    }

    let attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
    let total_dims = dim_names.len();
    for (i, name) in dim_names.iter().enumerate() {
        if let Some((first, last)) =
            get_cached_coord_bounds_with_rank(store.clone(), store_url, name, i, total_dims)
        {
            coordinates.insert(name.clone(), vec![first, last]);
        }
    }

    // Fallback: If dim_names contain generic "dim_i" names, query spatial coordinate bounds for lat and lon
    if coordinates.is_empty() || dim_names.iter().any(|d| d.starts_with("dim_")) {
        for candidate in &["lat", "latitude", "y", "lon", "longitude", "x"] {
            if let Some((first, last)) = get_cached_coord_bounds_with_rank(
                store.clone(),
                store_url,
                candidate,
                usize::MAX,
                total_dims,
            ) {
                coordinates.insert((*candidate).to_string(), vec![first, last]);
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
    ))
}

/// Fetches an arbitrary-rank hyperslab described by `request` with progress reporting.
pub fn fetch_block_with_progress(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
    on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
) -> Result<OctantBlock, BlockStoreError> {
    let dummy_store =
        super::generic_zarr::GenericZarrBlockStore::new(store.clone(), store_url, "zarr", "Zarr");
    let cached_array = dummy_store.get_or_open_array(&request.variable)?;
    fetch_block_from_cached_array(&cached_array, store, store_url, request, on_progress)
}
