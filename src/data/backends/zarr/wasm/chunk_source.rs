//! What the browser's Zarr and Icechunk stores share: both decode from an in-memory Zarr
//! store (`WasmZarrBlockStore`) and differ only in how they fetch chunks into it (by key,
//! or through Icechunk's manifests), so a block load and a variable's coordinates take the
//! same steps over either.

use zarrs::array::ArraySubset;

use super::WasmZarrBlockStore;
use super::partial_store::{Lease, array_prefixes};
use crate::data::backends::coord_bounds::CoordPreload;
use crate::data::blocks::{BlockRequest, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::utils::metadata::open_or_instantiate_array_normalized;

/// A browser store whose chunks are fetched into an in-memory Zarr store.
pub(crate) trait ChunkSource {
    /// The in-memory store blocks and coordinates decode from.
    fn zarr(&self) -> &WasmZarrBlockStore;

    /// Fetches the chunks `subset` of array `var` touches; the lease keeps them until the
    /// caller has decoded them, whatever other loads fetch meanwhile.
    async fn preload(
        &self,
        var: &str,
        subset: &ArraySubset,
        on_progress: ProgressCallback<'_>,
    ) -> Result<Lease, BlockStoreError>;
}

/// Loads one block: its chunks, its quantization scales' and its coordinates' fetched,
/// then decoded from memory.
pub(crate) async fn load_block(
    source: &impl ChunkSource,
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    let store = source.zarr();
    let variable = &request.slice.variable;
    let mut shape = (store
        .metadata
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref())
    .and_then(|meta| meta.variables.iter().find(|v| v.name == *variable))
    .map(|v| v.shape.clone())
    .unwrap_or_default();
    if shape.is_empty() {
        let path = format!("/{}", variable.trim_matches('/'));
        if let Ok(array) = open_or_instantiate_array_normalized(store.memory_store.clone(), &path) {
            shape = array.shape().to_vec();
        }
    }
    let subset = request.slice.to_array_subset(&shape)?;
    let _chunks = (source.preload(variable, &subset, on_progress).await)
        .inspect_err(|e| log::error!("[WASM] Failed downloading chunks for '{variable}': {e}"))?;
    // A quantised embedding block multiplies by its scale array's window.
    let _scales = match store.generic_store.scale_window(&request.slice) {
        Some((path, subset)) => Some(source.preload(&path, &subset, None).await?),
        None => None,
    };
    let _coords = preload_block_coordinates(source, &request.slice).await;
    store
        .fetch_block(&request.slice)
        .inspect_err(|e| log::error!("[WASM] Failed decoding block for '{variable}': {e}"))
}

/// Preloads the coordinate arrays a block for `request` reads (those of its range-selected
/// dimensions), so its coordinates decode from memory; the lease keeps them until the
/// block is decoded.
async fn preload_block_coordinates(
    source: &impl ChunkSource,
    request: &crate::data::slice_request::SliceRequest,
) -> Lease {
    let store = source.zarr();
    let arrays = super::coord_paths::block_coordinate_arrays(&store.memory_store, request);
    let lease = (store.partial_store).lease(Vec::new(), array_prefixes(&arrays));
    for (path, len) in arrays {
        let state = preload_coordinate_1d(source, &path, len).await;
        if state != CoordPreload::Complete {
            log::warn!("[WASM] Coordinate '{path}' {state:?}");
        }
    }
    lease
}

/// Fetches the coordinate arrays of `variable`'s dimensions (spatial first, at the root or
/// in its group) and reads them, keyed like `DatasetMetadata::dimension_coordinates`.
pub(crate) async fn load_variable_coordinates(
    source: &impl ChunkSource,
    variable: &crate::data::VariableInfo,
) -> std::collections::HashMap<String, crate::data::CoordValues> {
    use crate::data::backends::coord_bounds as cb;
    let store = source.zarr();
    let mut incomplete = Vec::new();
    let arrays = super::coord_paths::variable_coordinate_arrays(&store.memory_store, variable);
    // Kept until read below, whatever other loads fetch meanwhile.
    let _lease = (store.partial_store).lease(Vec::new(), array_prefixes(&arrays));
    for (path, len) in arrays {
        let state = preload_coordinate_1d(source, &path, len).await;
        if state != CoordPreload::Complete {
            incomplete.push((path, state));
        }
    }
    let mut coords = cb::fetch_all_dimension_coordinates_for_variables(
        store.memory_store.clone(),
        std::slice::from_ref(variable),
        Some(&store.base_url),
        None,
    );
    cb::settle_preloaded_coordinates(&mut coords, &incomplete);
    coords
}

/// Preloads every chunk of a 1D coordinate, which is read whole (`zarr::coords`): the
/// first and last chunks first, as the endpoints, then the rest.
#[allow(clippy::single_range_in_vec_init)]
async fn preload_coordinate_1d(source: &impl ChunkSource, path: &str, count: u64) -> CoordPreload {
    let Some(last) = count.checked_sub(1) else {
        return CoordPreload::Complete;
    };
    for ends in [0..1, last..count] {
        let subset = ArraySubset::new_with_ranges(&[ends]);
        if let Err(e) = Box::pin(source.preload(path, &subset, None)).await {
            log::error!("[WASM] Coordinate '{path}' endpoints failed to load: {e}");
            return CoordPreload::Failed;
        }
    }
    let subset = ArraySubset::new_with_ranges(&[0..count]);
    match Box::pin(source.preload(path, &subset, None)).await {
        Ok(_) => CoordPreload::Complete,
        Err(e) => {
            log::error!("[WASM] Coordinate '{path}' partly loaded: {e}");
            CoordPreload::Partial
        }
    }
}
