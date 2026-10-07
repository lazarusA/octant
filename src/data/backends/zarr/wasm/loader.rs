//! Async chunk preloading and progress reporting for WebAssembly Zarr.

use super::WasmZarrBlockStore;
use crate::data::backends::coord_bounds::CoordPreload;
use crate::data::backends::http::fetch_url_bytes;
use crate::data::blocks::{BlockRequest, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::utils::metadata::open_or_instantiate_array_normalized;
use futures::StreamExt;
use zarrs::array::ArraySubset;

/// Chunk requests kept in flight while preloading.
pub(crate) const CONCURRENT_FETCHES: usize = 64;

impl WasmZarrBlockStore {
    /// Computes and fetches all required chunk files for an array slice into memory.
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let clean_var = var_name.trim_matches('/');
        let array = open_or_instantiate_array_normalized(
            self.memory_store.clone(),
            &format!("/{clean_var}"),
        )
        .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;
        let missing = self.missing_chunks(&array, clean_var, subset);
        if !missing.is_empty() {
            log::info!(
                "[WASM Zarr] Preloading {} chunk(s) concurrently for '{clean_var}'",
                missing.len()
            );
            self.fetch_chunks(&missing, &mut on_progress).await?;
        }
        Ok(())
    }

    /// Store keys and URLs of the chunks of `subset` not yet in memory.
    fn missing_chunks<S: ?Sized + zarrs::storage::ReadableStorageTraits + 'static>(
        &self,
        array: &zarrs::array::Array<S>,
        clean_var: &str,
        subset: &ArraySubset,
    ) -> Vec<(String, String)> {
        let Ok(Some(chunks)) = array.chunks_in_array_subset(subset) else {
            return Vec::new();
        };
        chunks
            .indices()
            .into_iter()
            .filter_map(|indices| {
                let key = array.chunk_key(&indices).as_str().to_string();
                let key =
                    if clean_var.is_empty() || clean_var == "data" || key.starts_with(clean_var) {
                        key
                    } else {
                        format!("{clean_var}/{key}")
                    };
                let url = format!("{}/{}", self.base_url, key);
                (!self.has_key(&key)).then_some((key, url))
            })
            .collect()
    }

    /// Downloads `missing` `(key, url)` chunks with [`CONCURRENT_FETCHES`] requests in
    /// flight. A chunk the server does not have (HTTP 404) is sparse; other errors fail.
    async fn fetch_chunks(
        &self,
        missing: &[(String, String)],
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let mut fetches = futures::stream::iter(missing)
            .map(|(key, url)| async move { (key, url, fetch_url_bytes(url).await) })
            .buffer_unordered(CONCURRENT_FETCHES);
        while let Some((key, url, res)) = fetches.next().await {
            match res {
                Ok(chunk_bytes) => {
                    self.insert_key_bytes(key, &chunk_bytes)?;
                    if let Some(cb) = on_progress.as_mut() {
                        cb(chunk_bytes.len() as u64);
                    }
                }
                Err(err) if err.contains("HTTP 404") => {
                    log::warn!("[WASM Zarr] Chunk not found (HTTP 404 / sparse chunk): {url}");
                }
                Err(err) => {
                    log::error!("[WASM Zarr] Failed to fetch chunk '{url}': {err}");
                    return Err(format!("Failed to fetch chunk '{url}': {err}").into());
                }
            }
        }
        Ok(())
    }

    /// Preloads the coordinate arrays a block for `request` reads (those of its
    /// range-selected dimensions), so its coordinates decode from memory.
    pub async fn preload_block_coordinates(
        &self,
        request: &crate::data::slice_request::SliceRequest,
    ) {
        for (path, len) in super::coord_paths::block_coordinate_arrays(&self.memory_store, request)
        {
            let state = self.preload_coordinate_chunks_1d(&path, len).await;
            if state != CoordPreload::Complete {
                log::warn!("[WASM Zarr] Coordinate '{path}' {state:?}");
            }
        }
    }
}

/// Asynchronously loads one block on WASM, preloading required chunk bytes over HTTP if needed.
pub async fn load_one_wasm_with_progress(
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    let source_uri = &request.store.source().uri;

    // Route Icechunk data sources to Icechunk WASM loader
    if request.store.source().kind == crate::data::DataSourceKind::RemoteIcechunk
        || request.store.source().kind == crate::data::DataSourceKind::LocalIcechunk
        || source_uri.starts_with("icechunk+")
    {
        return crate::data::backends::icechunk::wasm::load_one_icechunk_wasm_with_progress(
            request,
            on_progress,
        )
        .await;
    }

    // If it's a procedural dataset or non-HTTP store, fetch directly
    if source_uri.starts_with("procedural://") || source_uri.starts_with("test://") {
        return request
            .store
            .fetch_with_progress(&request.slice, on_progress);
    }

    let clean_url = source_uri.trim_end_matches('/');
    let store = WasmZarrBlockStore::get_or_create(clean_url);

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
            open_or_instantiate_array_normalized(store.memory_store.clone(), &var_path)
        {
            shape = array.shape().to_vec();
        }
    }

    let subset = request.slice.to_array_subset(&shape);
    log::info!(
        "[WASM Zarr] Preloading chunks for '{}', subset: {:?}",
        request.slice.variable,
        subset.to_ranges()
    );

    // Asynchronously download any missing chunks for this slice
    if let Err(e) = store
        .preload_chunks_for_subset(&request.slice.variable, &subset, on_progress)
        .await
    {
        log::error!(
            "[WASM Zarr] Failed downloading chunks for '{}': {e}",
            request.slice.variable
        );
        return Err(e);
    }

    store.preload_block_coordinates(&request.slice).await;

    // Now decode the slice synchronously from in-memory chunks
    match store.fetch_block_with_progress(&request.slice, None) {
        Ok(block) => {
            log::info!(
                "[WASM Zarr] Successfully decoded block for '{}' ({} values, shape: {:?})",
                request.slice.variable,
                block.values.len(),
                block.shape
            );
            Ok(block)
        }
        Err(e) => {
            log::error!(
                "[WASM Zarr] Failed decoding block for '{}': {e}",
                request.slice.variable
            );
            Err(e)
        }
    }
}
