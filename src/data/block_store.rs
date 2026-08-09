//! Format-independent storage abstraction.
//!
//! Zarr, NetCDF, GeoTIFF, Icechunk, etc. implement this interface.
//! Higher layers never need to know about ReadableWritableListableStorage.
//!
//! This trait operates on bare `SliceRequest`s, not `BlockRequest` --
//! `BlockRequest` additionally carries the `StoreHandle` a request targets,
//! which is exactly what a `BlockStore` implementation *is*, so it would be
//! circular for the trait to take one.

use std::error::Error;

use super::{block_request::BlockResult, octant_block::OctantBlock, slice_request::SliceRequest};

pub type BlockStoreError = Box<dyn Error + Send + Sync>;

pub trait BlockStore: Send + Sync {
    /// Human-readable backend name.
    fn backend_name(&self) -> &str;

    /// List variables available from this source.
    fn variables(&self) -> Result<Vec<String>, BlockStoreError>;

    /// Load one arbitrary N-dimensional block.
    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError>;

    /// Default implementation for loading multiple variables from this
    /// same backend in one call.
    ///
    /// A backend may override this later to optimize shared/coordinated
    /// reads (e.g. several variables sharing chunks).
    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());

        for request in requests {
            blocks.push(self.fetch_block(request)?);
        }

        Ok(BlockResult::new(blocks))
    }
}
