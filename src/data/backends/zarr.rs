//! Zarr implementation of the generic BlockStore abstraction.

use super::zarr_storage;
use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};

pub struct ZarrBlockStore {
    storage: zarrs::storage::ReadableWritableListableStorage,
    source_url: String,
}

impl ZarrBlockStore {
    pub fn new(
        storage: zarrs::storage::ReadableWritableListableStorage,
        source_url: impl Into<String>,
    ) -> Self {
        Self {
            storage,
            source_url: source_url.into(),
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
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
        "zarr"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        // Variable/catalog discovery placeholder.
        Ok(Vec::new())
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        super::zarr_block::fetch_block(self.storage.clone(), &self.source_url, request)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());

        for request in requests {
            blocks.push(self.fetch_block(request)?);
        }

        Ok(BlockResult::new(blocks))
    }
}
