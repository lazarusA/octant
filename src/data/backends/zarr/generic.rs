//! Generic `GenericZarrBlockStore` implementation over `ReadableWritableListableStorage`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::block::{ArrayCachedState, fetch_block_from_cached_array, inherited_attributes};
pub use super::storage::{ZarrArrayHandle, open_raw_array};
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::storage::ReadableWritableListableStorage;

pub type CachedArrayEntry = Arc<ArrayCachedState>;

/// Default chunk cache size per array handle (32 MB).
pub const DEFAULT_CHUNK_CACHE_BYTES: u64 = 32 * 1024 * 1024;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    array_cache: RwLock<HashMap<String, Arc<ArrayCachedState>>>,
    chunk_cache_bytes: u64,
}

fn compute_chunk_cache_bytes(array: &ZarrArrayHandle, limit: u64) -> u64 {
    if limit != DEFAULT_CHUNK_CACHE_BYTES {
        return limit;
    }
    let rank = array.shape().len();
    let chunk_indices = vec![0; rank];
    let single_chunk_bytes = if let Ok(chunk_dims) = array.chunk_shape(&chunk_indices) {
        let elem_count = chunk_dims
            .iter()
            .try_fold(1u64, |acc, d| acc.checked_mul(d.get()))
            .unwrap_or(1);
        let elem_size = match array.data_type().size() {
            zarrs::array::DataTypeSize::Fixed(s) => (s as u64).max(1),
            zarrs::array::DataTypeSize::Variable => 4,
        };
        elem_count.saturating_mul(elem_size)
    } else {
        4 * 1024 * 1024
    };
    (single_chunk_bytes.saturating_mul(8)).clamp(64 * 1024 * 1024, 512 * 1024 * 1024)
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

    /// Gets an already-opened and cached `ArrayCachedState` or opens and caches it.
    pub fn get_or_open_array(
        &self,
        var_name: &str,
    ) -> Result<Arc<ArrayCachedState>, BlockStoreError> {
        let clean_name = var_name.trim_matches('/');
        let cache_guard = self.array_cache.read().unwrap_or_else(|p| p.into_inner());
        if let Some(cached) = cache_guard.get(clean_name) {
            return Ok(cached.clone());
        }
        drop(cache_guard);

        let raw_array = open_raw_array(&self.storage, clean_name)?;
        let cache_bytes = compute_chunk_cache_bytes(&raw_array, self.chunk_cache_bytes);
        let attributes = inherited_attributes(&raw_array, &self.storage, clean_name);
        let is_label_image = crate::data::ome::axes::is_label_image(&attributes);
        let ome_dim_names =
            crate::data::ome::axes::block_dimension_names(&attributes, raw_array.shape().len());

        let array_arc = Arc::new(raw_array);
        let chunk_cache = Arc::new(ChunkCacheDecodedLruSizeLimit::new(
            array_arc.clone(),
            cache_bytes,
        ));
        let cached = Arc::new(ArrayCachedState {
            array: array_arc,
            cache: chunk_cache,
            attributes,
            is_label_image,
            ome_dim_names,
        });

        let mut write_guard = self.array_cache.write().unwrap_or_else(|p| p.into_inner());
        write_guard.insert(clean_name.to_string(), cached.clone());
        Ok(cached)
    }

    /// Total number of opened array variables cached in this store handle.
    pub fn cached_arrays_count(&self) -> usize {
        self.array_cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .len()
    }

    /// Clears opened array handles and chunk caches.
    pub fn clear_array_cache(&self) {
        self.array_cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
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

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        let base_url = self.source_url.trim_end_matches('/');
        let default_name = format!("{}_store", self.backend_name);
        let dataset_name = base_url.split('/').next_back().unwrap_or(&default_name);

        let (variables, ome) = match crate::data::ome::discover_direct(self.storage.clone()) {
            Some(found) => {
                crate::data::ome::report_warnings(&found, dataset_name);
                (found.variables, Some(Arc::new(found.dataset)))
            }
            None => (
                crate::utils::extract_store_variables_consolidated(self.storage.clone(), base_url)
                    .map_err(|e| e.to_string())?,
                None,
            ),
        };

        Ok(DatasetMetadata {
            name: dataset_name.to_string(),
            store_type: self.store_type_label.to_string(),
            variables,
            dimension_coordinates: HashMap::new(),
            ome,
        })
    }

    fn variable_coordinates(
        &self,
        variable: &VariableInfo,
    ) -> Result<HashMap<String, CoordValues>, BlockStoreError> {
        let url = Some(self.source_url.trim_end_matches('/'));
        Ok(fetch_all_dimension_coordinates_for_variables(
            self.storage.clone(),
            std::slice::from_ref(variable),
            url,
        ))
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.fetch_block_with_progress(request, None)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        let cached = self.get_or_open_array(&request.variable)?;
        fetch_block_from_cached_array(
            &cached,
            self.storage.clone(),
            &self.source_url,
            request,
            on_progress,
        )
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rayon::prelude::*;
            requests
                .par_iter()
                .map(|r| self.fetch_block(r))
                .collect::<Result<_, _>>()
                .map(BlockResult::new)
        }
        #[cfg(target_arch = "wasm32")]
        requests
            .iter()
            .map(|r| self.fetch_block(r))
            .collect::<Result<_, _>>()
            .map(BlockResult::new)
    }
}
