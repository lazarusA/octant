//! Generic `GenericZarrBlockStore` implementation over `ReadableWritableListableStorage`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::block::{ArrayCachedState, fetch_block_from_cached_array, inherited_attributes};
use super::storage::{ZarrArrayHandle, open_raw_array};
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::storage::ReadableWritableListableStorage;

/// Decoded chunk bytes the open arrays of one store may hold together: the
/// least recently used arrays are closed beyond it (an OME-Zarr image opens
/// one array per level, each with its own chunk cache).
const OPEN_ARRAYS_BUDGET: u64 = 1024 * 1024 * 1024;

/// Open arrays with the size of their chunk caches, least recently used first.
type OpenArrays = Vec<(String, u64, Arc<ArrayCachedState>)>;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    open_arrays: Mutex<OpenArrays>,
    /// Held while an array is opened, so concurrent block fetches (prefetch
    /// workers) share one array and its chunk cache instead of each opening it.
    opening: Mutex<()>,
}

/// The chunk cache size of `array`: eight chunks, within 64-512 MB.
fn chunk_cache_bytes(array: &ZarrArrayHandle) -> u64 {
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
        Self {
            storage,
            source_url: source_url.into(),
            backend_name,
            store_type_label,
            open_arrays: Mutex::default(),
            opening: Mutex::default(),
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn storage(&self) -> &ReadableWritableListableStorage {
        &self.storage
    }

    /// The opened array `var_name` with its chunk cache, opened on first use.
    pub fn get_or_open_array(
        &self,
        var_name: &str,
    ) -> Result<Arc<ArrayCachedState>, BlockStoreError> {
        let clean_name = var_name.trim_matches('/');
        if let Some(open) = self.open_array(clean_name) {
            return Ok(open);
        }
        let _opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(open) = self.open_array(clean_name) {
            return Ok(open);
        }
        let (cached, cache_bytes) = self.open(clean_name)?;
        let mut open = self.open_arrays.lock().unwrap_or_else(|p| p.into_inner());
        open.push((clean_name.to_string(), cache_bytes, cached.clone()));
        while open.len() > 1 && open.iter().map(|(_, b, _)| b).sum::<u64>() > OPEN_ARRAYS_BUDGET {
            open.remove(0);
        }
        Ok(cached)
    }

    /// An array already open, marked most recently used.
    fn open_array(&self, name: &str) -> Option<Arc<ArrayCachedState>> {
        let mut open = self.open_arrays.lock().unwrap_or_else(|p| p.into_inner());
        let i = open.iter().position(|(n, _, _)| n == name)?;
        let entry = open.remove(i);
        let cached = entry.2.clone();
        open.push(entry);
        Some(cached)
    }

    /// The store's OME-Zarr variables with their discovery, else its plain
    /// Zarr variables.
    fn store_variables(
        &self,
    ) -> Result<(Vec<VariableInfo>, Option<crate::data::ome::Discovery>), BlockStoreError> {
        if let Some(mut found) = crate::data::ome::discover_direct(self.storage.clone()) {
            return Ok((std::mem::take(&mut found.variables), Some(found)));
        }
        let url = self.source_url.trim_end_matches('/');
        let vars = crate::utils::extract_store_variables_consolidated(self.storage.clone(), url)
            .map_err(|e| e.to_string())?;
        Ok((vars, None))
    }

    fn open(&self, name: &str) -> Result<(Arc<ArrayCachedState>, u64), BlockStoreError> {
        let raw_array = open_raw_array(&self.storage, name)?;
        let cache_bytes = chunk_cache_bytes(&raw_array);
        let reader = crate::data::ome::MetaReader::direct(self.storage.clone());
        let attributes = inherited_attributes(&raw_array, &reader, name);
        let ome = crate::data::ome::level::level_array(
            &reader,
            name,
            raw_array.shape().len(),
            raw_array.data_type(),
        );
        let array = Arc::new(raw_array);
        let cache = Arc::new(ChunkCacheDecodedLruSizeLimit::new(
            array.clone(),
            cache_bytes,
        ));
        let cached = ArrayCachedState {
            array,
            cache,
            attributes,
            ome,
        };
        Ok((Arc::new(cached), cache_bytes))
    }
}

impl BlockStore for GenericZarrBlockStore {
    fn backend_name(&self) -> &str {
        self.backend_name
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let (vars, _) = self.store_variables()?;
        Ok(vars.into_iter().map(|v| v.name).collect())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        let base_url = self.source_url.trim_end_matches('/');
        let default_name = format!("{}_store", self.backend_name);
        let dataset_name = base_url.split('/').next_back().unwrap_or(&default_name);

        let (variables, found) = self.store_variables()?;
        let ome = found.map(|found| {
            crate::data::ome::report_warnings(&found, dataset_name);
            Arc::new(found.dataset)
        });

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
