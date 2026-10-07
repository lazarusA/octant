//! Resolving and downloading the chunks of an Icechunk array subset in the browser:
//! manifests are fetched once, then chunks are fetched concurrently.

use std::sync::Arc;

use futures::StreamExt;
use icechunk_format::manifest::{ChunkPayload, Manifest, ManifestRef};
use icechunk_format::{ChunkIndices, NodeTag, ObjectId};
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableStorageTraits;

use super::header::decompress_icechunk_file;
use super::store::WasmIcechunkBlockStore;
use crate::data::backends::http::{fetch_url_byte_range, fetch_url_bytes};
use crate::data::backends::zarr::wasm::CONCURRENT_FETCHES;
use crate::data::blocks::{BlockStoreError, ProgressCallback};
use crate::utils::remote::s3_to_https;

type NodeId = ObjectId<8, NodeTag>;

impl WasmIcechunkBlockStore {
    /// Downloads every chunk of `subset` of `array` (at `clean_var`) not yet in memory,
    /// keeping [`CONCURRENT_FETCHES`] requests in flight. Chunks no manifest lists are
    /// sparse and read as the fill value.
    pub(super) async fn fetch_missing_chunks<S: ?Sized + ReadableStorageTraits + 'static>(
        &self,
        array: &Array<S>,
        clean_var: &str,
        subset: &ArraySubset,
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let (node_id, refs) = self.manifest_refs(clean_var)?;
        let manifests = self.load_manifests(&refs).await?;
        let (node_id, manifests) = (&node_id, &manifests);
        let missing = self.missing_chunks(array, subset);
        let mut fetches = futures::stream::iter(missing)
            .map(|(key, indices)| async move {
                let bytes = fetch_chunk(manifests, node_id, &indices, &self.base_url).await;
                (key, bytes)
            })
            .buffer_unordered(CONCURRENT_FETCHES);
        while let Some((key, bytes)) = fetches.next().await {
            match bytes? {
                Some(bytes) => {
                    self.inner.insert_key_bytes(&key, &bytes)?;
                    if let Some(cb) = on_progress.as_mut() {
                        cb(bytes.len() as u64);
                    }
                }
                None => log::debug!("[WASM Icechunk] Chunk {key} not in manifests (sparse fill)"),
            }
        }
        Ok(())
    }

    /// The node and manifest references recorded for array `clean_var`.
    fn manifest_refs(
        &self,
        clean_var: &str,
    ) -> Result<(NodeId, Vec<ManifestRef>), BlockStoreError> {
        let guard = self
            .array_manifests
            .read()
            .unwrap_or_else(|p| p.into_inner());
        let info = guard.get(clean_var).ok_or_else(|| {
            BlockStoreError::from(format!(
                "No manifest metadata recorded for array '{clean_var}'"
            ))
        })?;
        Ok((info.node_id.clone(), info.manifests.clone()))
    }

    /// The manifests of `refs`, newest (last appended) first, fetched once and cached.
    async fn load_manifests(
        &self,
        refs: &[ManifestRef],
    ) -> Result<Vec<Arc<Manifest>>, BlockStoreError> {
        let mut manifests = Vec::with_capacity(refs.len());
        for man_ref in refs.iter().rev() {
            let id = man_ref.object_id.to_string();
            let cached = self
                .cached_manifests
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .get(&id)
                .cloned();
            let manifest = match cached {
                Some(m) => m,
                None => {
                    let url = format!("{}/manifests/{id}", self.base_url);
                    log::info!("[WASM Icechunk] Fetching manifest: {url}");
                    let raw = fetch_url_bytes(&url)
                        .await
                        .map_err(|e| format!("Failed fetching manifest '{url}': {e}"))?;
                    let (_spec, decompressed) = decompress_icechunk_file(&raw)?;
                    let decoded = Manifest::from_buffer(decompressed)
                        .map_err(|e| format!("Failed parsing Icechunk manifest '{id}': {e:?}"))?;
                    let m = Arc::new(decoded);
                    self.cached_manifests
                        .write()
                        .unwrap_or_else(|p| p.into_inner())
                        .insert(id, m.clone());
                    m
                }
            };
            manifests.push(manifest);
        }
        Ok(manifests)
    }

    /// Store keys and chunk indices of the chunks of `subset` not yet in memory.
    fn missing_chunks<S: ?Sized + ReadableStorageTraits + 'static>(
        &self,
        array: &Array<S>,
        subset: &ArraySubset,
    ) -> Vec<(String, ChunkIndices)> {
        let Ok(Some(chunks)) = array.chunks_in_array_subset(subset) else {
            return Vec::new();
        };
        chunks
            .indices()
            .into_iter()
            .filter_map(|current| {
                let key = array.chunk_key(&current).as_str().to_string();
                let indices = ChunkIndices(current.iter().map(|&x| x as u32).collect());
                (!self.inner.has_key(&key)).then_some((key, indices))
            })
            .collect()
    }
}

/// The bytes of the chunk at `indices`, from the newest manifest that lists it; `None` for
/// sparse chunks.
async fn fetch_chunk(
    manifests: &[Arc<Manifest>],
    node_id: &NodeId,
    indices: &ChunkIndices,
    base_url: &str,
) -> Result<Option<Vec<u8>>, BlockStoreError> {
    for manifest in manifests {
        let (url, offset, length) = match manifest.get_chunk_payload(node_id, indices) {
            Ok(ChunkPayload::Virtual(v)) => (s3_to_https(v.location.url()), v.offset, v.length),
            Ok(ChunkPayload::Ref(r)) => (format!("{base_url}/chunks/{}", r.id), r.offset, r.length),
            Ok(ChunkPayload::Inline(bytes)) => return Ok(Some(bytes.to_vec())),
            _ => continue,
        };
        log::info!("[WASM Icechunk] Fetching chunk from '{url}' (offset: {offset}, len: {length})");
        let bytes = fetch_url_byte_range(&url, offset, length)
            .await
            .map_err(|e| format!("Failed fetching chunk from '{url}': {e}"))?;
        return Ok(Some(bytes));
    }
    Ok(None)
}
