//! Preloading the chunks of an Icechunk array subset, and of its 1D coordinates, into the
//! in-memory store the browser reads from.

use zarrs::array::ArraySubset;

use super::store::WasmIcechunkBlockStore;
use crate::data::backends::coord_bounds::CoordPreload;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
#[cfg(target_arch = "wasm32")]
use crate::utils::metadata::open_or_instantiate_array_normalized;

impl WasmIcechunkBlockStore {
    /// Preloads chunks required for a slice request by querying Icechunk manifests, then
    /// the 1D coordinates of the array's dimensions.
    #[cfg(target_arch = "wasm32")]
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let clean_var = var_name.trim_start_matches('/');
        let array = open_or_instantiate_array_normalized(
            self.inner.memory_store.clone(),
            &format!("/{clean_var}"),
        )
        .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;
        self.fetch_missing_chunks(&array, clean_var, subset, &mut on_progress)
            .await?;
        if array.shape().len() > 1 {
            for dim in crate::utils::resolve_array_dimension_names(&array) {
                let clean = dim.trim().trim_start_matches('/').to_string();
                if let Ok(coord_array) = open_or_instantiate_array_normalized(
                    self.inner.memory_store.clone(),
                    &format!("/{clean}"),
                ) && coord_array.shape().len() == 1
                {
                    let count = coord_array.shape().first().copied().unwrap_or(0);
                    let state = self.preload_coordinate_chunks_1d(&clean, count).await;
                    if state != CoordPreload::Complete {
                        log::warn!("[WASM Icechunk] Coordinate '{clean}' {state:?}");
                    }
                }
            }
        }
        Ok(())
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
            Ok(()) => CoordPreload::Complete,
            Err(_) => CoordPreload::Partial,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn preload_chunks_for_subset(
        &self,
        _var_name: &str,
        _subset: &ArraySubset,
        _on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        Ok(())
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
