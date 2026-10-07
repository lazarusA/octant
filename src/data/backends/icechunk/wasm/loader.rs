//! Asynchronous block loading routines for Icechunk WebAssembly data streaming.

#[cfg(target_arch = "wasm32")]
use super::store::WasmIcechunkBlockStore;
#[cfg(target_arch = "wasm32")]
use crate::data::blocks::BlockStore;
use crate::data::blocks::{BlockRequest, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
#[cfg(target_arch = "wasm32")]
use crate::utils::metadata::open_or_instantiate_array_normalized;

/// Asynchronously loads one block on WASM for an Icechunk dataset.
#[cfg(target_arch = "wasm32")]
pub async fn load_one_icechunk_wasm_with_progress(
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    let source_uri = &request.store.source().uri;
    let clean_url = source_uri
        .trim_start_matches("icechunk+")
        .trim_end_matches('/');
    let store = WasmIcechunkBlockStore::get_or_create(clean_url);

    let mut shape = request
        .store
        .inspect()
        .map(|meta| {
            meta.variables
                .iter()
                .find(|v| v.name == request.slice.variable)
                .map(|v| v.shape.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if shape.is_empty() {
        let clean_var = request.slice.variable.trim_matches('/');
        let var_path = format!("/{clean_var}");
        if let Ok(array) =
            open_or_instantiate_array_normalized(store.inner.memory_store.clone(), &var_path)
        {
            shape = array.shape().to_vec();
        }
    }

    let subset = request.slice.to_array_subset(&shape);
    log::info!(
        "[WASM Icechunk] Preloading chunks for '{}', subset: {:?}",
        request.slice.variable,
        subset.to_ranges()
    );

    // Asynchronously download any missing chunks for this slice via manifests
    if let Err(e) = store
        .preload_chunks_for_subset(&request.slice.variable, &subset, on_progress)
        .await
    {
        log::error!(
            "[WASM Icechunk] Failed downloading chunks for '{}': {e}",
            request.slice.variable
        );
        return Err(e);
    }

    store.preload_block_coordinates(&request.slice).await;

    // Decode slice synchronously from in-memory chunks
    match store.fetch_block_with_progress(&request.slice, None) {
        Ok(block) => {
            log::info!(
                "[WASM Icechunk] Successfully decoded block for '{}' ({} values, shape: {:?})",
                request.slice.variable,
                block.values.len(),
                block.shape
            );
            Ok(block)
        }
        Err(e) => {
            log::error!(
                "[WASM Icechunk] Failed decoding block for '{}': {e}",
                request.slice.variable
            );
            Err(e)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_one_icechunk_wasm_with_progress(
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    request
        .store
        .fetch_with_progress(&request.slice, on_progress)
}
