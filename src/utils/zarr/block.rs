//! Phase 2/3 of the OctantBlock redesign: a generic, N-dimensional fetch
//! that returns an `OctantBlock` instead of assuming a 2D matrix.
//!
//! Unlike `fetch_slice` (which hard-codes "first dim is time, last two are
//! lat/lon"), `fetch_block` takes an explicit `SliceRequest` — one
//! `DimensionSelection` per array dimension — and pulls exactly that
//! hyperslab, whatever its rank. This is what lets a single load serve a 2D
//! map, a 3D volume, or an animated sequence of either, without re-fetching.

use std::collections::HashMap;
use std::error::Error;

use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorage;

use crate::data::octant_block::OctantBlock;
use crate::utils::coordinates::get_cached_coord_bounds;

use super::slice::retrieve_array_subset_as_f32;
use super::{DimensionSelection, SliceRequest};

/// Fetches an arbitrary-rank hyperslab described by `request` and returns it
/// as a resident `OctantBlock`. No rendering assumptions are made here: it
/// is up to the caller (via `OctantBlock::matrix_slice` / `OctantBlock::volume`)
/// to decide later how to project it.
///
/// `request.selections` must have exactly one entry per array dimension —
/// same convention already used by `build_slice_request` in the variable
/// controls panel.
pub fn fetch_block(
    store: ReadableWritableListableStorage,
    store_url: &str,
    request: &SliceRequest,
) -> Result<OctantBlock, Box<dyn Error>> {
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

    let dim_names: Vec<String> = array
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

    // Resolve each DimensionSelection into a concrete [start, end) range,
    // clamped to the array's actual extent, and remember the block's shape
    // and origin so it can be reasoned about relative to the full array later
    // (sliding-window prefetch, Phase 8/9).
    let mut ranges: Vec<std::ops::Range<u64>> = Vec::with_capacity(rank);
    let mut block_shape = Vec::with_capacity(rank);
    let mut origin = Vec::with_capacity(rank);

    for (i, sel) in request.selections.iter().enumerate() {
        let dim_len = shape[i] as usize;
        let (start, end) = match sel {
            DimensionSelection::Index(idx) => (*idx, idx + 1),
            DimensionSelection::Range(r) => (r.start, r.end),
        };
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start + 1).min(dim_len);

        ranges.push(start as u64..end as u64);
        block_shape.push(end - start);
        origin.push(start);
    }

    let subset = ArraySubset::new_with_ranges(&ranges);
    let raw_values = retrieve_array_subset_as_f32(&array, &subset)?;

    let attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    let mut coordinates = HashMap::new();
    for name in &dim_names {
        if let Some((first, last)) = get_cached_coord_bounds(store.clone(), store_url, name) {
            coordinates.insert(name.clone(), vec![first, last]);
        }
    }

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
