//! Requests for resident N-dimensional data.
//!
//! Two levels, deliberately distinct:
//!
//! - `BlockRequest` is atomic: one variable/hyperslab selection against one
//!   already-opened `StoreHandle`. It carries its own store, so its cache
//!   identity (`cache_key()`) is fully self-contained and can be computed
//!   before any I/O happens.
//! - `BlockRequestBatch` is a collection of `BlockRequest`s belonging to
//!   one logical operation. It is NOT constrained to one dataset or one
//!   backend -- a batch can freely mix variables from many datasets, each
//!   opened against a different store.

use super::{
    block_cache::{BlockCache, BlockCacheKey},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
    store_handle::StoreHandle,
};

#[derive(Clone)]
pub struct BlockRequest {
    pub store: StoreHandle,
    pub slice: SliceRequest,
}

impl BlockRequest {
    pub fn new(store: StoreHandle, slice: SliceRequest) -> Self {
        Self { store, slice }
    }

    /// Cache/dedup identity, buildable before any I/O happens.
    pub fn cache_key(&self) -> BlockCacheKey {
        BlockCacheKey::new(self.store.source().id.clone(), &self.slice)
    }

    pub fn variable_name(&self) -> &str {
        &self.slice.variable
    }
}

/// A collection of `BlockRequest`s belonging to one logical operation
/// (e.g. "everything the UI currently needs resident").
#[derive(Clone, Default)]
pub struct BlockRequestBatch {
    requests: Vec<BlockRequest>,
}

impl BlockRequestBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            requests: Vec::with_capacity(capacity),
        }
    }

    pub fn from_requests(requests: Vec<BlockRequest>) -> Self {
        Self { requests }
    }

    pub fn push(&mut self, request: BlockRequest) {
        self.requests.push(request);
    }

    pub fn extend(&mut self, requests: impl IntoIterator<Item = BlockRequest>) {
        self.requests.extend(requests);
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn requests(&self) -> &[BlockRequest] {
        &self.requests
    }

    pub fn into_requests(self) -> Vec<BlockRequest> {
        self.requests
    }

    /// Requests not already resident in the cache.
    pub fn uncached(&self, cache: &BlockCache) -> Vec<BlockRequest> {
        self.requests
            .iter()
            .filter(|request| !cache.contains(&request.cache_key()))
            .cloned()
            .collect()
    }
}

/// Result of loading one or more blocks from a single `StoreHandle` call.
#[derive(Debug)]
pub struct BlockResult {
    pub blocks: Vec<OctantBlock>,
}

impl BlockResult {
    pub fn new(blocks: Vec<OctantBlock>) -> Self {
        Self { blocks }
    }

    pub fn into_inner(self) -> Vec<OctantBlock> {
        self.blocks
    }
}
