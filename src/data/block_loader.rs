//! Synchronous orchestration for loading one or more blocks.
//!
//! `BlockLoader` is cache-agnostic: callers are expected to filter a
//! `BlockRequestBatch` down to `uncached()` requests via `BlockCache`
//! before calling here (see `block_request.rs`). That keeps this a pure
//! "go get this data" step, with no cache side effects of its own.

use std::collections::HashMap;

use super::{
    block_cache::BlockCacheKey,
    block_request::{BlockRequest, BlockRequestBatch},
    block_store::BlockStoreError,
    octant_block::OctantBlock,
};

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
        on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        request
            .store
            .fetch_with_progress(&request.slice, on_progress)
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
