//! Preloading the chunks of an Icechunk array subset, and of its 1D coordinates, into the
//! in-memory store the browser reads from.

use zarrs::array::ArraySubset;

use super::store::WasmIcechunkBlockStore;
use crate::data::backends::coord_bounds::CoordPreload;
use crate::data::backends::zarr::wasm::partial_store::Lease;
#[cfg(target_arch = "wasm32")]
use crate::data::backends::zarr::wasm::partial_store::array_prefixes;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
#[cfg(target_arch = "wasm32")]
use crate::utils::metadata::open_or_instantiate_array_normalized;

impl WasmIcechunkBlockStore {
    /// Preloads chunks required for a slice request by querying Icechunk manifests; the
    /// lease keeps them until the caller has decoded them.
    #[cfg(target_arch = "wasm32")]
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<Lease, BlockStoreError> {
        let clean_var = var_name.trim_start_matches('/');
        let array = open_or_instantiate_array_normalized(
            self.inner.memory_store.clone(),
            &format!("/{clean_var}"),
        )
        .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;
        let keys = match array.chunks_in_array_subset(subset) {
            Ok(Some(chunks)) => chunks
                .indices()
                .into_iter()
                .map(|indices| array.chunk_key(&indices))
                .collect(),
            _ => Vec::new(),
        };
        let lease = self.inner.partial_store.lease(keys, Vec::new());
        self.fetch_missing_chunks(&array, clean_var, subset, &mut on_progress)
            .await?;
        Ok(lease)
    }

    /// Preloads the coordinate arrays a block for `request` reads (those of its
    /// range-selected dimensions), so its coordinates decode from memory; the lease keeps them
    /// until the block is decoded.
    #[cfg(target_arch = "wasm32")]
    pub async fn preload_block_coordinates(
        &self,
        request: &crate::data::slice_request::SliceRequest,
    ) -> Lease {
        use crate::data::backends::zarr::wasm::coord_paths::block_coordinate_arrays;
        let arrays = block_coordinate_arrays(&self.inner.memory_store, request);
        let lease = self
            .inner
            .partial_store
            .lease(Vec::new(), array_prefixes(&arrays));
        for (path, len) in arrays {
            let state = self.preload_coordinate_chunks_1d(&path, len).await;
            if state != CoordPreload::Complete {
                log::warn!("[WASM Icechunk] Coordinate '{path}' {state:?}");
            }
        }
        lease
    }

    /// Preloads every chunk of a 1D coordinate, which is read whole (`zarr::coords`): the
    /// first and last chunks first, as the endpoints, then the rest.
    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_coordinate_chunks_1d(&self, coord_name: &str, count: u64) -> CoordPreload {
        let Some(last) = count.checked_sub(1) else {
            return CoordPreload::Complete;
        };
        for ends in [0..1, last..count] {
            let subset = ArraySubset::new_with_ranges(&[ends]);
            if Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None))
                .await
                .is_err()
            {
                return CoordPreload::Failed;
            }
        }
        let subset = ArraySubset::new_with_ranges(&[0..count]);
        match Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None)).await {
            Ok(_) => CoordPreload::Complete,
            Err(_) => CoordPreload::Partial,
        }
    }

    /// Fetches the coordinate arrays of `variable`'s dimensions (spatial first, at the root
    /// or in its group) and reads them, keyed like `DatasetMetadata::dimension_coordinates`.
    #[cfg(target_arch = "wasm32")]
    pub async fn load_variable_coordinates(
        &self,
        variable: &crate::data::VariableInfo,
    ) -> std::collections::HashMap<String, crate::data::CoordValues> {
        use crate::data::backends::coord_bounds as cb;
        use crate::data::backends::zarr::wasm::coord_paths::variable_coordinate_arrays;
        let mut incomplete = Vec::new();
        let arrays = variable_coordinate_arrays(&self.inner.memory_store, variable);
        // Kept until read below.
        let _lease = self
            .inner
            .partial_store
            .lease(Vec::new(), array_prefixes(&arrays));
        for (path, len) in arrays {
            let state = self.preload_coordinate_chunks_1d(&path, len).await;
            if state != CoordPreload::Complete {
                incomplete.push((path, state));
            }
        }
        let mut coords = cb::fetch_all_dimension_coordinates_for_variables(
            self.inner.memory_store.clone(),
            std::slice::from_ref(variable),
            Some(&self.base_url),
            None,
        );
        cb::settle_preloaded_coordinates(&mut coords, &incomplete);
        coords
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn preload_chunks_for_subset(
        &self,
        _var_name: &str,
        _subset: &ArraySubset,
        _on_progress: ProgressCallback<'_>,
    ) -> Result<Lease, BlockStoreError> {
        Ok(self.inner.partial_store.lease(Vec::new(), Vec::new()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn preload_coordinate_chunks_1d(
        &self,
        _coord_name: &str,
        _count: u64,
    ) -> CoordPreload {
        CoordPreload::Complete
    }
}
