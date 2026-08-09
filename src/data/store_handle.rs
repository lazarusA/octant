//! Open handle to a selected data source.
//!
//! The UI selects a DataSource first. StoreHandle then opens the appropriate
//! backend and can be used for one or many variables from that source.

use std::sync::Arc;

use super::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    data_source::DataSource,
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};

#[derive(Clone)]
pub struct StoreHandle {
    source: DataSource,
    backend: Arc<dyn BlockStore>,
}

impl StoreHandle {
    pub fn new(source: DataSource, backend: Arc<dyn BlockStore>) -> Self {
        Self { source, backend }
    }

    pub fn source(&self) -> &DataSource {
        &self.source
    }

    pub fn backend(&self) -> &dyn BlockStore {
        self.backend.as_ref()
    }

    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    pub fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.backend.variables()
    }

    pub fn fetch(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.backend.fetch_block(request)
    }

    /// Fetches several variables/selections from this same store in one
    /// call, letting the backend optimize a coordinated read if it wants
    /// to.
    ///
    /// All requests here must target this store. A batch spanning multiple
    /// stores should go through `BlockRequestBatch` + `BlockLoader`
    /// instead, which groups by store before calling this per group.
    pub fn fetch_many(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        self.backend.fetch_blocks(requests)
    }
}
