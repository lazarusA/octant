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

    /// Inspect variables and metadata from this dataset source.
    fn inspect(&self) -> Result<super::metadata::DatasetMetadata, BlockStoreError> {
        let vars = self.variables()?;
        let var_infos = vars
            .into_iter()
            .map(|name| super::metadata::VariableInfo {
                name,
                data_type: "float32".to_string(),
                ..Default::default()
            })
            .collect();

        Ok(super::metadata::DatasetMetadata {
            name: self.backend_name().to_string(),
            store_type: self.backend_name().to_string(),
            variables: var_infos,
            dimension_coordinates: std::collections::HashMap::new(),
        })
    }

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
