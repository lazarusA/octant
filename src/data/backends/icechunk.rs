//! Icechunk implementation of the generic BlockStore abstraction.

#[cfg(not(target_arch = "wasm32"))]
use super::generic_zarr::GenericZarrBlockStore;
#[cfg(not(target_arch = "wasm32"))]
use super::icechunk_storage::build_sync_icechunk_store;
#[cfg(not(target_arch = "wasm32"))]
use crate::data::block_request::BlockResult;
use crate::data::{
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};

#[cfg(not(target_arch = "wasm32"))]
pub struct IcechunkBlockStore {
    inner: GenericZarrBlockStore,
}

#[cfg(not(target_arch = "wasm32"))]
impl IcechunkBlockStore {
    pub fn new(
        storage: zarrs::storage::ReadableWritableListableStorage,
        source_url: impl Into<String>,
    ) -> Self {
        Self {
            inner: GenericZarrBlockStore::new(storage, source_url, "icechunk", "Icechunk"),
        }
    }

    pub fn source_url(&self) -> &str {
        self.inner.source_url()
    }

    pub fn open(location: &str) -> Result<Self, BlockStoreError> {
        let storage = build_sync_icechunk_store(location).map_err(|e| e.to_string())?;
        Ok(Self::new(storage, location.trim_end_matches('/')))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BlockStore for IcechunkBlockStore {
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
        on_progress: crate::data::block_store::ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block_with_progress(request, on_progress)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        self.inner.fetch_blocks(requests)
    }
}

#[cfg(target_arch = "wasm32")]
pub struct IcechunkBlockStore;

#[cfg(target_arch = "wasm32")]
impl IcechunkBlockStore {
    pub fn open(_location: &str) -> Result<Self, BlockStoreError> {
        Err(
            "Icechunk repositories are only supported on the native desktop version of Octant."
                .into(),
        )
    }
}

#[cfg(target_arch = "wasm32")]
impl BlockStore for IcechunkBlockStore {
    fn backend_name(&self) -> &str {
        "Icechunk"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Err("Icechunk is not supported on web".into())
    }

    fn fetch_block_with_progress(
        &self,
        _request: &SliceRequest,
        _on_progress: crate::data::block_store::ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        Err("Icechunk is not supported on web".into())
    }
}
