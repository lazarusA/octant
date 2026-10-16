//! Sharded Icechunk arrays in the browser: each shard is a chunk the
//! manifests place at a span of a larger object (or inline), so its index
//! and the window's inner chunks are read by range within that span, as
//! for Zarr (`zarr/wasm/shard_fetch.rs`), never the whole shard.

use icechunk_format::ChunkIndices;
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableStorageTraits;

use super::chunks::{ChunkSource, locate_chunk};
use super::store::WasmIcechunkBlockStore;
use crate::data::backends::zarr::wasm::shard_fetch::ShardRef;
use crate::data::backends::zarr::wasm::shard_plan::ShardIndexSpec;
use crate::data::blocks::{BlockStoreError, ProgressCallback};

impl WasmIcechunkBlockStore {
    /// Fetches the index and `subset`'s inner chunks of every shard of
    /// `array` (at `clean_var`) it touches; shards no manifest lists are
    /// sparse.
    pub(super) async fn fetch_shard_parts<S: ?Sized + ReadableStorageTraits + 'static>(
        &self,
        array: &Array<S>,
        clean_var: &str,
        subset: &ArraySubset,
        spec: &ShardIndexSpec,
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let Ok(Some(chunks)) = array.chunks_in_array_subset(subset) else {
            return Ok(());
        };
        let grid: Vec<Vec<u64>> = chunks.indices().into_iter().map(|i| i.to_vec()).collect();
        let indices: Vec<ChunkIndices> = grid.iter().map(|g| chunk_indices(g)).collect();
        let (node_id, manifests) = self.manifests_holding(clean_var, &indices).await?;
        let mut shards = Vec::with_capacity(grid.len());
        for (shard, at) in grid.into_iter().zip(&indices) {
            let key = array.chunk_key(&shard).as_str().to_string();
            match locate_chunk(&manifests, &node_id, at, &self.base_url) {
                None => {}
                Some(ChunkSource::Inline(bytes)) => {
                    if !self.inner.has_key(&key) {
                        self.inner.insert_key_bytes(&key, &bytes)?;
                    }
                }
                Some(ChunkSource::Range {
                    url,
                    offset,
                    length,
                }) => shards.push(ShardRef {
                    key,
                    url,
                    indices: shard,
                    span: Some(offset..offset + length),
                }),
            }
        }
        self.inner
            .fetch_shard_parts(spec, &shards, &subset.to_ranges(), on_progress)
            .await
    }
}

/// Icechunk's indices of the chunk at grid position `grid`.
fn chunk_indices(grid: &[u64]) -> ChunkIndices {
    ChunkIndices(grid.iter().map(|&x| x as u32).collect())
}
