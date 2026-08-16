//! Zarr implementation of the generic BlockStore abstraction.

use super::{generic_zarr::GenericZarrBlockStore, zarr_storage};
use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};

pub struct ZarrBlockStore {
    inner: GenericZarrBlockStore,
}

impl ZarrBlockStore {
    pub fn new(
        storage: zarrs::storage::ReadableWritableListableStorage,
        source_url: impl Into<String>,
    ) -> Self {
        Self {
            inner: GenericZarrBlockStore::new(storage, source_url, "zarr", "Zarr"),
        }
    }

    pub fn source_url(&self) -> &str {
        self.inner.source_url()
    }

    pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
        let storage = zarr_storage::open_local_storage(path)?;
        Ok(Self::new(storage, path))
    }

    pub fn open_remote(url: &str) -> Result<Self, BlockStoreError> {
        let storage = zarr_storage::build_sync_store(url)?;
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

    fn inspect(&self) -> Result<crate::data::DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block(request)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block_with_progress(request, on_progress)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        self.inner.fetch_blocks(requests)
    }
}
