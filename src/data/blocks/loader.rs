//! Synchronous orchestration for loading one or more blocks.
//!
//! `BlockLoader` is cache-agnostic: callers are expected to filter a
//! `BlockRequestBatch` down to `uncached()` requests via `BlockCache`
//! before calling here (see `block_request.rs`). That keeps this a pure
//! "go get this data" step, with no cache side effects of its own.

use std::collections::HashMap;

use super::cache::BlockCacheKey;
use super::request::{BlockRequest, BlockRequestBatch};
use super::store::BlockStoreError;
use crate::data::octant_block::OctantBlock;

/// Outcome of loading a single requested block. Keyed so results from a
/// batch can be matched back up to their cache key regardless of
/// completion order or individual failure.
pub struct BlockLoadOutcome {
    pub key: BlockCacheKey,
    pub result: Result<OctantBlock, BlockStoreError>,
}

/// Results of a `BlockRequestBatch`, one outcome per request. A failure on
/// one request never discards the others.
pub struct BlockBatchOutcome {
    pub outcomes: Vec<BlockLoadOutcome>,
}

impl BlockBatchOutcome {
    pub fn successful_count(&self) -> usize {
        self.outcomes.iter().filter(|o| o.result.is_ok()).count()
    }

    pub fn failed_count(&self) -> usize {
        self.outcomes.iter().filter(|o| o.result.is_err()).count()
    }
}

pub struct BlockLoader;

impl BlockLoader {
    /// Loads a single block through its own store.
    pub fn load_one(request: &BlockRequest) -> Result<OctantBlock, BlockStoreError> {
        request.store.fetch(&request.slice)
    }

    /// Loads a single block through its own store, reporting progressive bytes downloaded.
    pub fn load_one_with_progress(
        request: &BlockRequest,
        on_progress: super::store::ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        request
            .store
            .fetch_with_progress(&request.slice, on_progress)
    }

    /// The browser's [`load_one_with_progress`](Self::load_one_with_progress):
    /// each format's async loader fetches the block's chunks, then decodes them.
    #[cfg(target_arch = "wasm32")]
    pub async fn load_one_wasm(
        request: &BlockRequest,
        on_progress: super::store::ProgressCallback<'_>,
    ) -> Result<OctantBlock, BlockStoreError> {
        use crate::data::DataSourceKind as Kind;
        use crate::data::backends::{geotiff, icechunk, zarr};
        match request.store.source().kind {
            Kind::RemoteIcechunk | Kind::LocalIcechunk => {
                icechunk::wasm::load_one_icechunk_wasm_with_progress(request, on_progress).await
            }
            Kind::RemoteGeoTiff | Kind::LocalGeoTiff => {
                geotiff::wasm::load_one_geotiff_wasm_with_progress(request, on_progress).await
            }
            Kind::Procedural => request
                .store
                .fetch_with_progress(&request.slice, on_progress),
            _ => zarr::load_one_wasm_with_progress(request, on_progress).await,
        }
    }

    /// Loads every request in a batch.
    pub fn load_batch(batch: &BlockRequestBatch) -> BlockBatchOutcome {
        let mut grouped: HashMap<String, Vec<&BlockRequest>> = HashMap::new();

        for request in batch.requests() {
            grouped
                .entry(request.store.source().id.clone())
                .or_default()
                .push(request);
        }

        let mut outcomes = Vec::with_capacity(batch.len());

        for requests in grouped.into_values() {
            let Some(first) = requests.first() else {
                continue;
            };

            let slices: Vec<_> = requests.iter().map(|r| r.slice.clone()).collect();

            match first.store.fetch_many(&slices) {
                Ok(result) => {
                    for (request, block) in requests.iter().zip(result.into_inner()) {
                        outcomes.push(BlockLoadOutcome {
                            key: request.cache_key(),
                            result: Ok(block),
                        });
                    }
                }
                Err(_) => {
                    for request in requests {
                        outcomes.push(BlockLoadOutcome {
                            key: request.cache_key(),
                            result: Self::load_one(request),
                        });
                    }
                }
            }
        }

        BlockBatchOutcome { outcomes }
    }
}
