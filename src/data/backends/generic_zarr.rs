//! Generic BlockStore implementation over any zarrs ReadableWritableListableStorage.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};
use zarrs::array::Array;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::storage::{ReadableWritableListableStorage, ReadableWritableListableStorageTraits};

pub type ZarrArrayHandle = Array<dyn ReadableWritableListableStorageTraits>;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    array_cache: RwLock<HashMap<String, Arc<ZarrArrayHandle>>>,
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
            array_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn storage(&self) -> &ReadableWritableListableStorage {
        &self.storage
    }

    /// Gets an already-opened `ZarrArrayHandle` or opens and caches it.
    pub fn get_or_open_array(
        &self,
        var_name: &str,
    ) -> Result<Arc<ZarrArrayHandle>, BlockStoreError> {
        let clean_name = var_name.trim_start_matches('/');
        let cache_guard = self.array_cache.read().unwrap_or_else(|p| p.into_inner());
        if let Some(cached) = cache_guard.get(clean_name) {
            return Ok(cached.clone());
        }
        drop(cache_guard);

        let var_path = if var_name.starts_with('/') {
            var_name.to_string()
        } else {
            format!("/{}", var_name)
        };

        let raw_array = if let Ok(arr) = Array::open(self.storage.clone(), &var_path) {
            arr
        } else if let Ok(group) = Group::open(self.storage.clone(), "/")
            && let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata()
            && let Some(node_meta) = metadata.get(clean_name).or_else(|| metadata.get(&var_path))
            && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
                self.storage.clone(),
                &var_path,
                node_meta,
            )
        {
            arr
        } else if var_name == "data" || var_name.is_empty() {
            Array::open(self.storage.clone(), "/")?
        } else {
            Array::open(self.storage.clone(), &var_path)?
        };

        let array_arc = Arc::new(raw_array);

        let mut write_guard = self.array_cache.write().unwrap_or_else(|p| p.into_inner());
        write_guard.insert(clean_name.to_string(), array_arc.clone());

        Ok(array_arc)
    }

    /// Total number of opened array variables cached in this store handle.
    pub fn cached_arrays_count(&self) -> usize {
        let cache_guard = self.array_cache.read().unwrap_or_else(|p| p.into_inner());
        cache_guard.len()
    }

    /// Clears opened array handles.
    pub fn clear_array_cache(&self) {
        let mut write_guard = self.array_cache.write().unwrap_or_else(|p| p.into_inner());
        write_guard.clear();
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
        self.fetch_block_with_progress(request, None)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        let array = self.get_or_open_array(&request.variable)?;
        super::zarr_block::fetch_block_from_cached_array(
            &array,
            self.storage.clone(),
            &self.source_url,
            request,
            on_progress,
        )
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        use rayon::prelude::*;
        let blocks: Result<Vec<OctantBlock>, BlockStoreError> = requests
            .par_iter()
            .map(|request| self.fetch_block(request))
            .collect();
        Ok(BlockResult::new(blocks?))
    }
}
