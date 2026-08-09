//! Generic, N-dimensional fetch that returns an `OctantBlock`.

use std::collections::HashMap;

use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorage;

use crate::data::block_store::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::utils::coordinates::get_cached_coord_bounds;

use super::slice::retrieve_array_subset_as_f32;

/// Fetches an arbitrary-rank hyperslab described by `request` and returns it
/// as a resident `OctantBlock`.
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
        if let Some((first, last)) = crate::utils::coordinates::get_cached_coord_bounds_with_rank(
            store.clone(),
            store_url,
            name,
            i,
            total_dims,
        ) {
            coordinates.insert(name.clone(), vec![first, last]);
        }
    }

    let raw_values = crate::utils::grid::check_and_orient_block_grid(
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
