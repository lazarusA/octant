//! Generic BlockStore implementation over any zarrs ReadableWritableListableStorage.

use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};
use zarrs::storage::ReadableWritableListableStorage;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
}

impl GenericZarrBlockStore {
    pub fn new(
        storage: ReadableWritableListableStorage,
        source_url: impl Into<String>,
        backend_name: &'static str,
        store_type_label: &'static str,
    ) -> Self {
        Self {
            storage,
            source_url: source_url.into(),
            backend_name,
            store_type_label,
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn storage(&self) -> &ReadableWritableListableStorage {
        &self.storage
    }
}

impl BlockStore for GenericZarrBlockStore {
    fn backend_name(&self) -> &str {
        self.backend_name
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
        let base_url = self.source_url.trim_end_matches('/');
        let variables =
            crate::utils::extract_store_variables_consolidated(self.storage.clone(), base_url)
                .map_err(|e| e.to_string())?;

        let dim_names: Vec<String> = variables
            .iter()
            .flat_map(|v| v.dimension_names.clone())
            .collect();
        let dimension_coordinates = crate::utils::fetch_all_dimension_coordinates(
            self.storage.clone(),
            &dim_names,
            Some(base_url),
        );

        let default_name = format!("{}_store", self.backend_name);
        let dataset_name = base_url
            .split('/')
            .next_back()
            .unwrap_or(&default_name)
            .to_string();

        Ok(crate::data::DatasetMetadata {
            name: dataset_name,
            store_type: self.store_type_label.to_string(),
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
