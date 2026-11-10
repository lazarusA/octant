//! Preloading the chunks of an Icechunk array subset into the in-memory store the browser
//! reads from (`ChunkSource`).

use zarrs::array::ArraySubset;

use super::store::WasmIcechunkBlockStore;
use crate::data::backends::zarr::WasmZarrBlockStore;
use crate::data::backends::zarr::wasm::chunk_source::ChunkSource;
use crate::data::backends::zarr::wasm::partial_store::Lease;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
use crate::utils::metadata::open_or_instantiate_array_normalized;

impl WasmIcechunkBlockStore {
    /// Preloads chunks required for a slice request by querying Icechunk manifests; the
    /// lease keeps them until the caller has decoded them.
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
}

impl ChunkSource for WasmIcechunkBlockStore {
    fn zarr(&self) -> &WasmZarrBlockStore {
        &self.inner
    }

    async fn preload(
        &self,
        var: &str,
        subset: &ArraySubset,
        on_progress: ProgressCallback<'_>,
    ) -> Result<Lease, BlockStoreError> {
        self.preload_chunks_for_subset(var, subset, on_progress)
            .await
    }
}
