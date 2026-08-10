//! Generic, N-dimensional fetch that returns an `OctantBlock` from a Zarr array.

use std::collections::HashMap;
use zarrs::storage::ReadableWritableListableStorage;

#[cfg(not(target_arch = "wasm32"))]
use zarrs::array::{Array, ArraySubset};

#[cfg(not(target_arch = "wasm32"))]
use super::zarr_slice::retrieve_array_subset_as_f32;
use crate::data::block_store::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::coordinates::get_cached_coord_bounds_with_rank;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::grid::check_and_orient_block_grid;

/// Fetches an arbitrary-rank hyperslab described by `request` and returns it
/// as a resident `OctantBlock`.
#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_block(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    let var_path = if request.variable.starts_with('/') {
        request.variable.clone()
    } else {
        format!("/{}", request.variable)
    };

    let array = Array::open(store.clone(), &var_path)?;
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
        .unwrap_or_else(|| (0..rank).map(|i| format!("dim_{i}")).collect());

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
    let raw_values = retrieve_array_subset_as_f32(&array, &subset).map_err(|e| e.to_string())?;

    let attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    let mut coordinates = HashMap::new();
    let total_dims = dim_names.len();
    for (i, name) in dim_names.iter().enumerate() {
        if let Some((first, last)) =
            get_cached_coord_bounds_with_rank(store.clone(), store_url, name, i, total_dims)
        {
            coordinates.insert(name.clone(), vec![first, last]);
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

#[cfg(target_arch = "wasm32")]
pub async fn fetch_block_wasm_async(
    store_url: &str,
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    use std::sync::Arc;
    use zarrs_object_store::AsyncObjectStore;
    use object_store::http::HttpBuilder;
    use zarrs::array::{Array, ArraySubset};
    use super::zarr_slice::retrieve_array_subset_as_f32_async;

    let clean_url = store_url.trim_end_matches('/');
    let http_store = HttpBuilder::new()
        .with_url(clean_url)
        .build()
        .map_err(|e| e.to_string())?;

    let async_store = Arc::new(AsyncObjectStore::new(http_store));

    let var_path = if request.variable.starts_with('/') {
        request.variable.clone()
    } else {
        format!("/{}", request.variable)
    };

    let array = Array::open_async(async_store.clone(), &var_path)
        .await
        .map_err(|e| e.to_string())?;

    let shape = array.shape();
    let rank = shape.len();

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
        .unwrap_or_else(|| (0..rank).map(|i| format!("dim_{i}")).collect());

    let mut ranges: Vec<std::ops::Range<u64>> = Vec::with_capacity(rank);
    let mut block_shape = Vec::with_capacity(rank);
    let mut origin = Vec::with_capacity(rank);

    for (i, sel) in request.selections.iter().enumerate() {
        let dim_len = shape.get(i).copied().unwrap_or(1) as usize;
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
    let raw_values = retrieve_array_subset_as_f32_async(&array, &subset)
        .await
        .map_err(|e| e.to_string())?;

    let attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    Ok(OctantBlock::new(
        request.variable.clone(),
        block_shape,
        dim_names,
        origin,
        raw_values,
        HashMap::new(),
        attributes,
    ))
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_block(
    _store: ReadableWritableListableStorage,
    _store_url: &str,
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    let mut block_shape = Vec::new();
    let mut origin = Vec::new();
    for sel in &request.selections {
        let (start, end) = match sel {
            DimensionSelection::Index(idx) => (*idx, idx.saturating_add(1)),
            DimensionSelection::Range { start, end } => (*start, *end),
        };
        block_shape.push((end - start).max(1));
        origin.push(start);
    }
    if block_shape.is_empty() {
        block_shape = vec![64, 64];
        origin = vec![0, 0];
    }
    let total_elems: usize = block_shape.iter().product();
    let mut raw_values = Vec::with_capacity(total_elems);
    let t_offset = origin.first().copied().unwrap_or(0) as f32 * 0.1;
    for i in 0..total_elems {
        let x = (i + origin[0]) as f32 / block_shape[0].max(1) as f32;
        raw_values.push((x * 6.28 + t_offset).sin());
    }
    let dim_names = match block_shape.len() {
        1 => vec!["x".to_string()],
        2 => vec!["y".to_string(), "x".to_string()],
        3 => vec!["z".to_string(), "y".to_string(), "x".to_string()],
        _ => (0..block_shape.len()).map(|i| format!("dim_{i}")).collect(),
    };
    Ok(OctantBlock::new(
        request.variable.clone(),
        block_shape,
        dim_names,
        origin,
        raw_values,
        HashMap::new(),
        HashMap::new(),
    ))
}
