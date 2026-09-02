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
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::storage::{ReadableStorage, ReadableStorageTraits, ReadableWritableListableStorage};

pub type ZarrArrayHandle = Array<dyn ReadableStorageTraits>;
pub type CachedArrayEntry = (Arc<ZarrArrayHandle>, Arc<ChunkCacheDecodedLruSizeLimit>);

/// Default chunk cache size per array handle (32 MB).
pub const DEFAULT_CHUNK_CACHE_BYTES: u64 = 32 * 1024 * 1024;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    array_cache: RwLock<HashMap<String, CachedArrayEntry>>,
    chunk_cache_bytes: u64,
}

impl GenericZarrBlockStore {
    pub fn new(
        storage: ReadableWritableListableStorage,
        source_url: impl Into<String>,
        backend_name: &'static str,
        store_type_label: &'static str,
    ) -> Self {
        Self::with_chunk_cache_limit(
            storage,
            source_url,
            backend_name,
            store_type_label,
            DEFAULT_CHUNK_CACHE_BYTES,
        )
    }

    pub fn with_chunk_cache_limit(
        storage: ReadableWritableListableStorage,
        source_url: impl Into<String>,
        backend_name: &'static str,
        store_type_label: &'static str,
        chunk_cache_bytes: u64,
    ) -> Self {
        Self {
            storage,
            source_url: source_url.into(),
            backend_name,
            store_type_label,
            array_cache: RwLock::new(HashMap::new()),
            chunk_cache_bytes,
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn storage(&self) -> &ReadableWritableListableStorage {
        &self.storage
    }

    /// Gets an already-opened `(ZarrArrayHandle, ChunkCacheDecodedLruSizeLimit)` or opens and caches it.
    pub fn get_or_open_array(
        &self,
        var_name: &str,
    ) -> Result<(Arc<ZarrArrayHandle>, Arc<ChunkCacheDecodedLruSizeLimit>), BlockStoreError> {
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

        let readable_store: ReadableStorage = self.storage.clone();

        let raw_array = if let Ok(arr) = Array::open(readable_store.clone(), &var_path) {
            arr
        } else if let Ok(group) = Group::open(self.storage.clone(), "/")
            && let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata()
            && let Some(node_meta) = metadata.get(clean_name).or_else(|| metadata.get(&var_path))
            && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
                readable_store.clone(),
                &var_path,
                node_meta,
            )
        {
            arr
        } else if var_name == "data" || var_name.is_empty() {
            Array::open(readable_store.clone(), "/")?
        } else {
            Array::open(readable_store.clone(), &var_path)?
        };

        let rank = raw_array.shape().len();
        let chunk_indices = vec![0; rank];
        let single_chunk_bytes = if let Ok(chunk_dims) = raw_array.chunk_shape(&chunk_indices) {
            let elem_count = chunk_dims
                .iter()
                .try_fold(1u64, |acc, d| acc.checked_mul(d.get()))
                .unwrap_or(1);
            let elem_size = match raw_array.data_type().size() {
                zarrs::array::DataTypeSize::Fixed(s) => (s as u64).max(1),
                zarrs::array::DataTypeSize::Variable => 4,
            };
            elem_count.saturating_mul(elem_size)
        } else {
            4 * 1024 * 1024
        };

        // Cache at least 8 full chunks per variable, clamped between 64 MB and 512 MB
        let dynamic_cache_bytes = if self.chunk_cache_bytes != DEFAULT_CHUNK_CACHE_BYTES {
            self.chunk_cache_bytes
        } else {
            single_chunk_bytes
                .saturating_mul(8)
                .clamp(64 * 1024 * 1024, 512 * 1024 * 1024)
        };

        let array_arc = Arc::new(raw_array);
        let chunk_cache = Arc::new(ChunkCacheDecodedLruSizeLimit::new(
            array_arc.clone(),
            dynamic_cache_bytes,
        ));
        let tuple = (array_arc, chunk_cache);

        let mut write_guard = self.array_cache.write().unwrap_or_else(|p| p.into_inner());
        write_guard.insert(clean_name.to_string(), tuple.clone());

        Ok(tuple)
    }

    /// Total number of opened array variables cached in this store handle.
    pub fn cached_arrays_count(&self) -> usize {
        let cache_guard = self.array_cache.read().unwrap_or_else(|p| p.into_inner());

        cache_guard.len()
    }

    /// Clears opened array handles and chunk caches.
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
        let (array, cache) = self.get_or_open_array(&request.variable)?;
        super::zarr_block::fetch_block_from_cached_array(
            &array,
            &cache,
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
