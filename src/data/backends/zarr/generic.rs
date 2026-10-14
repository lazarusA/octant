//! Generic `GenericZarrBlockStore` implementation over `ReadableWritableListableStorage`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::block::{ArrayCachedState, fetch_block_from_cached_array, inherited_attributes};
use super::storage::{ZarrArrayHandle, open_raw_array, root_consolidated};
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;
use zarrs::storage::ReadableWritableListableStorage;

/// Decoded chunk bytes the open arrays of one store may hold together: the
/// least recently used arrays are closed beyond it (an OME-Zarr image opens
/// one array per level, each with its own chunk cache).
#[cfg(not(target_arch = "wasm32"))]
const OPEN_ARRAYS_BUDGET: u64 = 1024 * 1024 * 1024;
#[cfg(target_arch = "wasm32")]
const OPEN_ARRAYS_BUDGET: u64 = 256 * 1024 * 1024;

/// Open arrays with the size of their chunk caches, least recently used first.
type OpenArrays = Vec<(String, u64, Arc<ArrayCachedState>)>;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    open_arrays: Mutex<OpenArrays>,
    /// One lock per array being opened, so concurrent block fetches (prefetch
    /// workers) share one array and its chunk cache instead of each opening
    /// it, while different arrays open in parallel.
    opening: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    consolidated: OnceLock<ConsolidatedMetadataMetadata>,
}

/// The chunk cache size of `array`: eight chunks within 64-512 MB, at most all its chunks.
fn chunk_cache_bytes(array: &ZarrArrayHandle) -> u64 {
    let elem_size = match array.data_type().size() {
        zarrs::array::DataTypeSize::Fixed(s) => (s as u64).max(1),
        zarrs::array::DataTypeSize::Variable => 4,
    };
    let shape = array.shape();
    let Ok(chunk) = array.chunk_shape(&vec![0; shape.len()]) else {
        return 64 * 1024 * 1024;
    };
    // Every chunk decodes whole, so the array's size rounds up to chunks.
    let (mut one_chunk, mut whole) = (Some(elem_size), Some(elem_size));
    for (&d, c) in shape.iter().zip(chunk.iter()) {
        let c = c.get();
        one_chunk = one_chunk.and_then(|v| v.checked_mul(c));
        whole = whole.and_then(|v| v.checked_mul(d.div_ceil(c).checked_mul(c)?));
    }
    let (one_chunk, whole) = (one_chunk.unwrap_or(u64::MAX), whole.unwrap_or(u64::MAX));
    one_chunk
        .saturating_mul(8)
        .clamp(64 * 1024 * 1024, 512 * 1024 * 1024)
        .min(whole)
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
            consolidated: OnceLock::new(),
        }
    }

    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn storage(&self) -> &ReadableWritableListableStorage {
        &self.storage
    }

    /// The opened array `var_name` with its chunk cache, opened on first use.
    fn get_or_open_array(&self, var_name: &str) -> Result<Arc<ArrayCachedState>, BlockStoreError> {
        let name = var_name.trim_matches('/');
        if let Some(open) = self.open_array(name) {
            return Ok(open);
        }
        let slot = {
            let mut opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
            opening.entry(name.to_string()).or_default().clone()
        };
        let _opening = slot.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(open) = self.open_array(name) {
            return Ok(open);
        }
        let opened = self.open(name).map(|(cached, cache_bytes)| {
            let mut open = self.open_arrays.lock().unwrap_or_else(|p| p.into_inner());
            open.push((name.to_string(), cache_bytes, cached.clone()));
            while open.len() > 1 && open.iter().map(|(_, b, _)| b).sum::<u64>() > OPEN_ARRAYS_BUDGET
            {
                open.remove(0);
            }
            cached
        });
        // Removed once the array is listed open: later callers find it there.
        let mut opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
        opening.remove(name);
        opened
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

    /// The root's consolidated metadata, kept once found (a browser store fills in later).
    fn consolidated(&self) -> Option<&ConsolidatedMetadataMetadata> {
        if let Some(found) = self.consolidated.get() {
            return Some(found);
        }
        let found = root_consolidated(&self.storage)?;
        Some(self.consolidated.get_or_init(|| found))
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
        let raw_array = open_raw_array(&self.storage, name, self.consolidated())?;
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
        let cached = ArrayCachedState {
            cache: Arc::new(ChunkCacheDecodedLruSizeLimit::new(
                array.clone(),
                cache_bytes,
            )),
            array,
            attributes: Arc::new(attributes),
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
            &self.storage,
            &self.source_url,
            request,
            on_progress,
        )
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        #[cfg(not(target_arch = "wasm32"))]
        let blocks: Result<_, BlockStoreError> = {
            use rayon::prelude::*;
            requests.par_iter().map(|r| self.fetch_block(r)).collect()
        };
        #[cfg(target_arch = "wasm32")]
        let blocks: Result<_, BlockStoreError> =
            requests.iter().map(|r| self.fetch_block(r)).collect();
        blocks.map(BlockResult::new)
    }
}
