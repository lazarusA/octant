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
        let vars = crate::utils::extract_store_variables_consolidated(
            self.storage.clone(),
            &self.source_url,
        )
        .map_err(|e| e.to_string())?;
        Ok(vars.into_iter().map(|v| v.name).collect())
    }

    fn inspect(&self) -> Result<crate::data::DatasetMetadata, BlockStoreError> {
        let store_name = self
            .source_url
            .split('/')
            .next_back()
            .unwrap_or("zarr_store")
            .to_string();

        let variables = crate::utils::extract_store_variables_consolidated(
            self.storage.clone(),
            &self.source_url,
        )
        .map_err(|e| e.to_string())?;

        let dim_names: Vec<String> = variables
            .iter()
            .flat_map(|v| v.dimension_names.clone())
            .collect();
        let dimension_coordinates = crate::utils::fetch_all_dimension_coordinates(
            self.storage.clone(),
            &dim_names,
            Some(&self.source_url),
        );

        Ok(crate::data::DatasetMetadata {
            name: store_name,
            store_type: "Zarr".to_string(),
            variables,
            dimension_coordinates,
        })
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        super::zarr_block::fetch_block(self.storage.clone(), &self.source_url, request)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        super::zarr_block::fetch_block_with_progress(
            self.storage.clone(),
            &self.source_url,
            request,
            on_progress,
        )
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());

        for request in requests {
            blocks.push(self.fetch_block(request)?);
        }

        Ok(BlockResult::new(blocks))
    }
}
