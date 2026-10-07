//! Concrete `ZarrBlockStore` implementation over `ReadableWritableListableStorage`.

use super::generic::GenericZarrBlockStore;
use super::storage;
use std::collections::HashMap;

use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};
use zarrs::storage::ReadableWritableListableStorage;

pub struct ZarrBlockStore {
    inner: GenericZarrBlockStore,
}

impl ZarrBlockStore {
    pub fn new(storage: ReadableWritableListableStorage, source_url: impl Into<String>) -> Self {
        Self {
            inner: GenericZarrBlockStore::new(storage, source_url, "zarr", "Zarr"),
        }
    }

    pub fn source_url(&self) -> &str {
        self.inner.source_url()
    }

    pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
        let storage = storage::open_local_storage(path).map_err(|e| format!("{e}"))?;
        Ok(Self::new(storage, path))
    }

    pub fn open_remote(url: &str) -> Result<Self, BlockStoreError> {
        let storage = storage::build_sync_store(url).map_err(|e| format!("{e}"))?;
        Ok(Self::new(storage, url.trim_end_matches('/')))
    }
}

impl BlockStore for ZarrBlockStore {
    fn backend_name(&self) -> &str {
        self.inner.backend_name()
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.inner.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn variable_coordinates(
        &self,
        variable: &VariableInfo,
    ) -> Result<HashMap<String, CoordValues>, BlockStoreError> {
        self.inner.variable_coordinates(variable)
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block(request)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block_with_progress(request, on_progress)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        self.inner.fetch_blocks(requests)
    }
}
