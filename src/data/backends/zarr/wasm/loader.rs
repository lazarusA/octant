//! Async chunk preloading and progress reporting for WebAssembly Zarr.

use super::WasmZarrBlockStore;
use super::partial_store::Lease;
use crate::data::backends::http::{fetch_url_bytes, is_not_found};
use crate::data::blocks::{BlockStoreError, ProgressCallback, progress};
use crate::utils::metadata::open_or_instantiate_array_normalized;
use futures::StreamExt;
use zarrs::array::ArraySubset;
use zarrs::storage::StoreKey;

/// Chunk requests kept in flight while preloading.
pub(crate) const CONCURRENT_FETCHES: usize = 64;

impl WasmZarrBlockStore {
    /// Computes and fetches all required chunk files for an array slice into
    /// memory; the lease keeps them until the caller has decoded them,
    /// whatever other loads fetch meanwhile.
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<Lease, BlockStoreError> {
        let clean_var = var_name.trim_matches('/');
        let array = open_or_instantiate_array_normalized(
            self.memory_store.clone(),
            &format!("/{clean_var}"),
        )
        .map_err(|e| format!("Failed to open array '{clean_var}': {e}"))?;
        let shards = self.shards_of(&array, clean_var, subset);
        let keys = shards
            .iter()
            .filter_map(|s| StoreKey::new(&s.key).ok())
            .collect();
        let lease = self.partial_store.lease(keys, Vec::new());
        // A sharded array with a plain index: its index and the subset's
        // inner chunks, by range.
        if let Some(spec) = super::shard_plan::ShardIndexSpec::of_array(&array) {
            self.fetch_shard_parts(&spec, &shards, &subset.to_ranges(), &mut on_progress)
                .await?;
            return Ok(lease);
        }
        let missing: Vec<(String, String)> = shards
            .into_iter()
            .filter(|c| !self.has_key(&c.key))
            .map(|c| (c.key, c.url))
            .collect();
        if !missing.is_empty() {
            log::info!(
                "[WASM Zarr] Preloading {} chunk(s) concurrently for '{clean_var}'",
                missing.len()
            );
            self.fetch_chunks(&missing, &mut on_progress).await?;
        }
        Ok(lease)
    }

    /// The chunks (shards, for a sharded array) `subset` touches, with their
    /// store keys and URLs.
    fn shards_of<S: ?Sized + zarrs::storage::ReadableStorageTraits + 'static>(
        &self,
        array: &zarrs::array::Array<S>,
        clean_var: &str,
        subset: &ArraySubset,
    ) -> Vec<super::shard_fetch::ShardRef> {
        let Ok(Some(chunks)) = array.chunks_in_array_subset(subset) else {
            return Vec::new();
        };
        chunks
            .indices()
            .into_iter()
            .map(|indices| {
                let key = array.chunk_key(&indices).as_str().to_string();
                let key =
                    if clean_var.is_empty() || clean_var == "data" || key.starts_with(clean_var) {
                        key
                    } else {
                        format!("{clean_var}/{key}")
                    };
                let url = format!("{}/{}", self.base_url, key);
                super::shard_fetch::ShardRef {
                    key,
                    url,
                    indices: indices.to_vec(),
                    span: None,
                }
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
        progress::plan(missing.len() as u64);
        let mut fetches = futures::stream::iter(missing)
            .map(|(key, url)| async move { (key, url, fetch_url_bytes(url).await) })
            .buffer_unordered(CONCURRENT_FETCHES);
        while let Some((key, url, res)) = fetches.next().await {
            match res {
                Ok(chunk_bytes) => {
                    let len = chunk_bytes.len() as u64;
                    self.insert_chunk_bytes(key, chunk_bytes)?;
                    progress::arrived(1);
                    if let Some(cb) = on_progress.as_mut() {
                        cb(len);
                    }
                }
                Err(err) if is_not_found(&err) => {
                    progress::arrived(1);
                    log::warn!("[WASM Zarr] Chunk not found (sparse chunk): {url}");
                }
                Err(err) => {
                    log::error!("[WASM Zarr] Failed to fetch chunk '{url}': {err}");
                    return Err(format!("Failed to fetch chunk '{url}': {err}").into());
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl super::chunk_source::ChunkSource for WasmZarrBlockStore {
    fn zarr(&self) -> &WasmZarrBlockStore {
        self
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
