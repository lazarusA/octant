//! Async chunk preloading and progress reporting for WebAssembly Zarr.

use super::WasmZarrBlockStore;
use crate::data::backends::http::fetch_url_bytes;
use crate::data::blocks::{BlockRequest, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::utils::metadata::open_or_instantiate_array_normalized;
use zarrs::array::ArraySubset;

impl WasmZarrBlockStore {
    /// Computes and fetches all required chunk files for an array slice into memory.
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let clean_var = var_name.trim_matches('/');
        let var_path = format!("/{clean_var}");

        let array = open_or_instantiate_array_normalized(self.memory_store.clone(), &var_path)
            .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;

        let rank = array.shape().len();
        let zero_idx = vec![0u64; rank];

        let chunk_dims = array
            .chunk_shape(&zero_idx)
            .map_err(|e| format!("Failed to get chunk shape: {e}"))?;

        // Calculate chunk index ranges per dimension
        let mut chunk_ranges = Vec::with_capacity(rank);
        for (i, dim_len) in chunk_dims.iter().enumerate() {
            let c_len = dim_len.get();
            let sel_start = subset.start()[i];
            let sel_shape = subset.shape()[i];
            let sel_end = sel_start + sel_shape;

            let c_start = sel_start / c_len;
            let c_end = (sel_end.saturating_sub(1) / c_len) + 1;
            chunk_ranges.push(c_start..c_end);
        }

        // 1. Collect all missing chunk keys that need to be fetched
        let mut missing_chunks: Vec<(String, String)> = Vec::new();
        let mut current = chunk_ranges.iter().map(|r| r.start).collect::<Vec<u64>>();
        loop {
            let chunk_rel_key = array.chunk_key(&current);
            let store_key_str = chunk_rel_key.as_str().to_string();

            let full_chunk_key = if clean_var.is_empty()
                || clean_var == "data"
                || store_key_str.starts_with(clean_var)
            {
                store_key_str
            } else {
                format!("{clean_var}/{store_key_str}")
            };

            if !self.has_key(&full_chunk_key) {
                let chunk_url = format!("{}/{}", self.base_url, full_chunk_key);
                missing_chunks.push((full_chunk_key, chunk_url));
            }

            // Advance chunk indices
            let mut carry = true;
            for i in (0..rank).rev() {
                current[i] += 1;
                if current[i] < chunk_ranges[i].end {
                    carry = false;
                    break;
                }
                current[i] = chunk_ranges[i].start;
            }
            if carry {
                break;
            }
        }

        // 2. Concurrently download chunks in parallel batches
        if !missing_chunks.is_empty() {
            log::info!(
                "[WASM Zarr] Preloading {} chunk(s) concurrently for '{}'",
                missing_chunks.len(),
                clean_var
            );
            const CONCURRENT_BATCH_SIZE: usize = 32;
            for batch in missing_chunks.chunks(CONCURRENT_BATCH_SIZE) {
                let futures_batch: Vec<_> = batch
                    .iter()
                    .map(|(key, url)| async move { (key, url, fetch_url_bytes(url).await) })
                    .collect();

                let results = futures::future::join_all(futures_batch).await;
                for (key, url, res) in results {
                    match res {
                        Ok(chunk_bytes) => {
                            let bytes_len = chunk_bytes.len() as u64;
                            self.insert_key_bytes(key, &chunk_bytes)?;
                            if let Some(ref mut cb) = on_progress {
                                cb(bytes_len);
                            }
                        }
                        Err(err) if err.contains("HTTP 404") => {
                            log::warn!(
                                "[WASM Zarr] Chunk not found (HTTP 404 / sparse chunk): {url}"
                            );
                        }
                        Err(err) => {
                            log::error!("[WASM Zarr] Failed to fetch chunk '{url}': {err}");
                            return Err(format!("Failed to fetch chunk '{url}': {err}").into());
                        }
                    }
                }
            }
        }

        // Preload associated 1D coordinate arrays (e.g. lat, lon, time, cell_ids) for spatial bounds & axes
        if rank > 1 {
            let dim_names = crate::utils::resolve_array_dimension_names(&array);
            let group_prefix = clean_var.rfind('/').map(|idx| &clean_var[..idx]);
            for dim in dim_names {
                let clean = dim.trim().trim_start_matches('/').to_string();
                let mut paths = vec![format!("/{clean}")];
                if let Some(gp) = group_prefix {
                    paths.push(format!("/{gp}/{clean}"));
                }
                for coord_path in paths {
                    let target_name = coord_path.trim_start_matches('/').to_string();
                    if let Ok(coord_array) =
                        open_or_instantiate_array_normalized(self.memory_store.clone(), &coord_path)
                        && coord_array.shape().len() == 1
                    {
                        let count = coord_array.shape().first().copied().unwrap_or(0);
                        self.preload_coordinate_chunks_1d(&target_name, count).await;
                        break;
                    }
                }
            }
        }

        Ok(())
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
