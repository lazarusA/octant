//! Generic `GenericZarrBlockStore` implementation over `ReadableWritableListableStorage`.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::block::{
    ArrayCachedState, ArrayReader, fetch_block_from_cached_array, inherited_attributes,
    selection_window,
};
use super::open_arrays::{OpenArrays, chunk_cache_bytes};
use super::quantization::{self, PixelScale, Quantization};
use super::storage::{open_raw_array, root_consolidated};
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;
use zarrs::array::{ArrayShardedReadableExtCache, ArraySubset};
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;
use zarrs::storage::ReadableWritableListableStorage;

/// What a sharded array's open state counts against the open arrays'
/// budget: its shard indexes, not decoded chunks.
const SHARD_INDEX_BYTES: u64 = 4 * 1024 * 1024;

pub struct GenericZarrBlockStore {
    storage: ReadableWritableListableStorage,
    source_url: String,
    backend_name: &'static str,
    store_type_label: &'static str,
    open_arrays: OpenArrays,
    consolidated: OnceLock<Option<ConsolidatedMetadataMetadata>>,
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
            open_arrays: OpenArrays::default(),
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
        self.open_arrays
            .get_or_open(var_name.trim_matches('/'), |name| self.open(name))
    }

    /// The root's consolidated metadata, read once on the desktop; a browser
    /// store fills in later, so there only a found one is kept.
    fn consolidated(&self) -> Option<&ConsolidatedMetadataMetadata> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.consolidated
                .get_or_init(|| root_consolidated(&self.storage))
                .as_ref()
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(found) = self.consolidated.get().and_then(Option::as_ref) {
                return Some(found);
            }
            let found = root_consolidated(&self.storage)?;
            self.consolidated.get_or_init(|| Some(found)).as_ref()
        }
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
        let mut vars =
            crate::utils::extract_store_variables_consolidated(self.storage.clone(), url)
                .map_err(|e| e.to_string())?;
        crate::data::multiscale::geozarr::link_pyramids_in(self.storage.clone(), &mut vars);
        crate::data::crs::attach_zarr_grid_mappings_in(self.storage.clone(), &mut vars);
        Ok((vars, None))
    }

    /// The per-pixel scale of embedding array `name` (of `shape`), opened.
    fn pixel_scale(
        &self,
        name: &str,
        shape: &[u64],
        attributes: &HashMap<String, String>,
    ) -> Option<PixelScale> {
        let path = quantization::scale_path(attributes, name, shape)?;
        let scales = self
            .get_or_open_array(&path)
            .inspect_err(|e| log::warn!("[Zarr] '{name}' keeps its quantised values: {e}"))
            .ok()?;
        let dim = quantization::embedding_dim(attributes, shape, scales.array.shape())?;
        Some(PixelScale { path, dim })
    }

    /// The scale array and window a block for `request` multiplies by, for
    /// the browser to fetch alongside the block's chunks.
    pub fn scale_window(&self, request: &SliceRequest) -> Option<(String, ArraySubset)> {
        let cached = self.get_or_open_array(&request.variable).ok()?;
        let q = cached.quantization.as_ref()?.scale_array()?;
        let (origin, shape) = selection_window(request, cached.array.shape()).ok()?;
        Some((q.path.clone(), q.subset(&origin, &shape)))
    }

    fn open(&self, name: &str) -> Result<(Arc<ArrayCachedState>, u64), BlockStoreError> {
        let raw_array = open_raw_array(&self.storage, name, self.consolidated())?;
        let sharded = zarrs::array::ArrayShardedExt::is_sharded(&raw_array);
        // A sharded array keeps shard indexes, not decoded shards.
        let cache_bytes = if sharded {
            SHARD_INDEX_BYTES
        } else {
            chunk_cache_bytes(&raw_array)
        };
        let reader = crate::data::ome::MetaReader::direct(self.storage.clone());
        let attributes = inherited_attributes(&raw_array, &reader, name);
        let quantization = self
            .pixel_scale(name, raw_array.shape(), &attributes)
            .map(Quantization::PerPixel)
            .or_else(|| quantization::value_quantization(&attributes, raw_array.shape()));
        let ome = crate::data::ome::level::level_array(
            &reader,
            name,
            raw_array.shape().len(),
            raw_array.data_type(),
        );
        let dims = crate::utils::resolve_array_dimension_names(&raw_array);
        let affine = crate::data::multiscale::geozarr_layout::array_placement(
            &reader,
            name,
            (&dims, raw_array.shape()),
            &attributes,
        );
        let array = Arc::new(raw_array);
        let reader = if sharded {
            ArrayReader::Sharded(ArrayShardedReadableExtCache::new(&array))
        } else {
            let cache = ChunkCacheDecodedLruSizeLimit::new(array.clone(), cache_bytes);
            ArrayReader::Cached(Arc::new(cache))
        };
        let cached = ArrayCachedState {
            reader,
            array,
            attributes: Arc::new(attributes),
            ome,
            quantization,
            affine,
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
        let scales = cached
            .quantization
            .as_ref()
            .and_then(Quantization::scale_array)
            .map(|q| self.get_or_open_array(&q.path))
            .transpose()?;
        fetch_block_from_cached_array(
            &cached,
            scales.as_deref(),
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
