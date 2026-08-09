//! Zarr implementation of the generic BlockStore abstraction.

use std::collections::HashMap;

use zarrs::array::{Array, ArraySubset};

use super::zarr_storage;
use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    octant_block::OctantBlock,
    slice_request::{DimensionSelection, SliceRequest},
};

use super::zarr_slice::retrieve_array_subset_as_f32;

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

    fn variable_path(variable: &str) -> String {
        if variable.starts_with('/') {
            variable.to_string()
        } else {
            format!("/{variable}")
        }
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
        let variable_path = Self::variable_path(&request.variable);

        let array = Array::open(self.storage.clone(), &variable_path)?;

        let shape = array.shape();
        let rank = shape.len();

        if request.selections.len() != rank {
            return Err(format!(
                "variable '{}' has rank {}, but request has {} selections",
                request.variable,
                rank,
                request.selections.len()
            )
            .into());
        }

        let dimension_names: Vec<String> = array
            .dimension_names()
            .as_ref()
            .map(|names| {
                names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| name.clone().unwrap_or_else(|| format!("dim_{i}")))
                    .collect()
            })
            .unwrap_or_else(|| (0..rank).map(|i| format!("dim_{i}")).collect());

        let mut ranges = Vec::with_capacity(rank);
        let mut block_shape = Vec::with_capacity(rank);
        let mut origin = Vec::with_capacity(rank);

        for (dimension, selection) in request.selections.iter().enumerate() {
            let dimension_len = shape[dimension] as usize;

            if dimension_len == 0 {
                return Err(format!(
                    "variable '{}' contains an empty dimension {}",
                    request.variable, dimension
                )
                .into());
            }

            let (requested_start, requested_end) = match selection {
                DimensionSelection::Index(index) => (*index, index.saturating_add(1)),

                DimensionSelection::Range { start, end } => (*start, *end),
            };

            let start = requested_start.min(dimension_len - 1);

            let end = requested_end
                .max(start.saturating_add(1))
                .min(dimension_len);

            ranges.push(start as u64..end as u64);
            block_shape.push(end - start);
            origin.push(start);
        }

        let subset = ArraySubset::new_with_ranges(&ranges);

        let values = retrieve_array_subset_as_f32(&array, &subset).map_err(|e| e.to_string())?;

        let attributes: HashMap<String, String> = array
            .attributes()
            .iter()
            .map(|(key, value)| (key.clone(), value.to_string()))
            .collect();

        Ok(OctantBlock::new(
            request.variable.clone(),
            block_shape,
            dimension_names,
            origin,
            values,
            HashMap::new(),
            attributes,
        ))
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());

        for request in requests {
            blocks.push(self.fetch_block(request)?);
        }

        Ok(BlockResult::new(blocks))
    }
}
