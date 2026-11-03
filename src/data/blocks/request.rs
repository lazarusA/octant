//! Requests for resident N-dimensional data.
//!
//! A `BlockRequest` is one variable/hyperslab selection against one
//! already-opened `StoreHandle`. It carries its own store, so its cache
//! identity (`cache_key()`) is self-contained and can be computed before
//! any I/O happens.

use super::cache::BlockCacheKey;
use crate::data::{StoreHandle, slice_request::SliceRequest};

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
