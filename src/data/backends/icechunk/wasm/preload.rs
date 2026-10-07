#[cfg(target_arch = "wasm32")]
use icechunk_format::ChunkIndices;
#[cfg(target_arch = "wasm32")]
use icechunk_format::manifest::{ChunkPayload, Manifest};
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
use zarrs::array::ArraySubset;

#[cfg(target_arch = "wasm32")]
use super::header::decompress_icechunk_file;
use super::store::WasmIcechunkBlockStore;
#[cfg(target_arch = "wasm32")]
use crate::data::backends::http::{fetch_url_byte_range, fetch_url_bytes};
use crate::data::blocks::{BlockStoreError, ProgressCallback};
#[cfg(target_arch = "wasm32")]
use crate::utils::metadata::open_or_instantiate_array_normalized;
#[cfg(target_arch = "wasm32")]
use crate::utils::remote::s3_to_https;

impl WasmIcechunkBlockStore {
    /// Preloads chunks required for a slice request by querying Icechunk manifests.
    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let clean_var = var_name.trim_start_matches('/');
        let var_path = format!("/{clean_var}");

        let array =
            open_or_instantiate_array_normalized(self.inner.memory_store.clone(), &var_path)
                .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;

        let rank = array.shape().len();
        let zero_idx = vec![0u64; rank];

        let chunk_dims = array
            .chunk_shape(&zero_idx)
            .map_err(|e| format!("Failed to get chunk shape: {e}"))?;

        // Retrieve manifest metadata for this array
        let (node_id, manifests) = {
            let guard = self
                .array_manifests
                .read()
                .unwrap_or_else(|p| p.into_inner());
            let info = guard.get(clean_var).ok_or_else(|| {
                BlockStoreError::from(format!(
                    "No manifest metadata recorded for array '{clean_var}'"
                ))
            })?;
            (info.node_id.clone(), info.manifests.clone())
        };

        // Calculate chunk index ranges per dimension
        let mut chunk_ranges = Vec::with_capacity(rank);
        for (i, dim_len) in chunk_dims.iter().enumerate() {
            let c_len = dim_len.get().max(1);
            let sel_start = subset.start()[i];
            let sel_shape = subset.shape()[i];
            let sel_end = sel_start + sel_shape;

            let c_start = sel_start / c_len;
            let c_end = (sel_end.saturating_sub(1) / c_len) + 1;
            chunk_ranges.push(c_start..c_end);
        }

        // Iterate over Cartesian product of chunk indices
        let mut current = chunk_ranges.iter().map(|r| r.start).collect::<Vec<u64>>();

        loop {
            let chunk_rel_key = array.chunk_key(&current);
            let store_key_str = chunk_rel_key.as_str();

            if !self.inner.has_key(store_key_str) {
                let chunk_indices = ChunkIndices(current.iter().map(|&x| x as u32).collect());
                let mut chunk_resolved = false;

                // Search manifests in reverse order (newest/appended manifests first)
                for man_ref in manifests.iter().rev() {
                    let man_id_str = man_ref.object_id.to_string();

                    // Check manifest cache or fetch & decompress
                    let cached_opt = {
                        let guard = self
                            .cached_manifests
                            .read()
                            .unwrap_or_else(|p| p.into_inner());
                        guard.get(&man_id_str).cloned()
                    };

                    let manifest_arc = match cached_opt {
                        Some(m) => m,
                        None => {
                            let man_url = format!("{}/manifests/{man_id_str}", self.base_url);
                            log::info!("[WASM Icechunk] Fetching manifest: {man_url}");

                            let raw_man_bytes = fetch_url_bytes(&man_url).await.map_err(|e| {
                                format!("Failed fetching manifest '{man_url}': {e}")
                            })?;

                            let (_spec, decomp_man) = decompress_icechunk_file(&raw_man_bytes)?;
                            let decoded_man = Manifest::from_buffer(decomp_man).map_err(|e| {
                                format!("Failed parsing Icechunk manifest '{man_id_str}': {e:?}")
                            })?;

                            let arc_m = Arc::new(decoded_man);
                            let mut guard = self
                                .cached_manifests
                                .write()
                                .unwrap_or_else(|p| p.into_inner());
                            guard.insert(man_id_str.clone(), arc_m.clone());
                            arc_m
                        }
                    };

                    let chunk_bytes_opt = match manifest_arc
                        .get_chunk_payload(&node_id, &chunk_indices)
                    {
                        Ok(ChunkPayload::Virtual(vchunk)) => {
                            let location_url = vchunk.location.url();
                            let target_url = s3_to_https(location_url);
                            log::info!(
                                "[WASM Icechunk] Fetching virtual chunk from '{target_url}' (offset: {}, len: {})",
                                vchunk.offset,
                                vchunk.length
                            );
                            let bytes =
                                fetch_url_byte_range(&target_url, vchunk.offset, vchunk.length)
                                    .await
                                    .map_err(|e| {
                                        format!(
                                            "Failed fetching virtual chunk from '{target_url}': {e}"
                                        )
                                    })?;
                            Some(bytes)
                        }
                        Ok(ChunkPayload::Ref(rchunk)) => {
                            let target_url = format!("{}/chunks/{}", self.base_url, rchunk.id);
                            log::info!(
                                "[WASM Icechunk] Fetching chunk ref from '{target_url}' (offset: {}, len: {})",
                                rchunk.offset,
                                rchunk.length
                            );
                            let bytes =
                                fetch_url_byte_range(&target_url, rchunk.offset, rchunk.length)
                                    .await
                                    .map_err(|e| {
                                        format!(
                                            "Failed fetching chunk ref from '{target_url}': {e}"
                                        )
                                    })?;
                            Some(bytes)
                        }
                        Ok(ChunkPayload::Inline(bytes)) => Some(bytes.to_vec()),
                        _ => None,
                    };

                    if let Some(chunk_bytes) = chunk_bytes_opt {
                        let bytes_len = chunk_bytes.len() as u64;
                        self.inner.insert_key_bytes(store_key_str, &chunk_bytes)?;
                        if let Some(ref mut cb) = on_progress {
                            cb(bytes_len);
                        }
                        chunk_resolved = true;
                        break;
                    }
                }

                if !chunk_resolved {
                    log::debug!(
                        "[WASM Icechunk] Chunk {store_key_str} not in manifests (sparse fill)"
                    );
                }
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

        // Preload coordinate chunks
        if rank > 1 {
            let dim_names = crate::utils::resolve_array_dimension_names(&array);
            for dim in dim_names {
                let clean = dim.trim().trim_start_matches('/').to_string();
                let coord_path = format!("/{clean}");
                if let Ok(coord_array) = open_or_instantiate_array_normalized(
                    self.inner.memory_store.clone(),
                    &coord_path,
                ) && coord_array.shape().len() == 1
                {
                    let count = coord_array.shape().first().copied().unwrap_or(0);
                    self.preload_coordinate_chunks_1d(&clean, count).await;
                }
            }
        }

        Ok(())
    }

    /// Preloads every chunk of a 1D coordinate, which is read whole (`zarr::coords`).
    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_coordinate_chunks_1d(&self, coord_name: &str, count: u64) {
        if count == 0 {
            return;
        }
        let subset = ArraySubset::new_with_ranges(&[0..count]);
        let _ = Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None)).await;
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
    pub async fn preload_coordinate_chunks_1d(&self, _coord_name: &str, _count: u64) {}
}
