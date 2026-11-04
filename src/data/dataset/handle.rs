//! Open handle to a selected data source.
//!
//! The UI selects a DataSource first. StoreHandle then opens the appropriate
//! backend and can be used for one or many variables from that source.

use std::collections::HashMap;
use std::sync::Arc;

use super::source::DataSource;
use crate::data::{
    blocks::{BlockStore, BlockStoreError},
    metadata::{CoordValues, DatasetMetadata, VariableInfo},
    octant_block::OctantBlock,
    slice_request::SliceRequest,
};

#[derive(Clone)]
pub struct StoreHandle {
    source: DataSource,
    backend: Arc<dyn BlockStore>,
}

impl StoreHandle {
    pub fn new(source: DataSource, backend: Arc<dyn BlockStore>) -> Self {
        Self { source, backend }
    }

    pub fn source(&self) -> &DataSource {
        &self.source
    }

    pub fn backend(&self) -> &dyn BlockStore {
        self.backend.as_ref()
    }

    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    pub fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.backend.variables()
    }

    pub fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        let mut meta = self.backend.inspect()?;
        crate::data::crs::assign(&mut meta);
        Ok(meta)
    }

    /// The coordinates of `variable`'s dimensions; see `BlockStore::variable_coordinates`.
    pub fn variable_coordinates(
        &self,
        variable: &VariableInfo,
    ) -> Result<HashMap<String, CoordValues>, BlockStoreError> {
        self.backend.variable_coordinates(variable)
    }

    pub fn fetch(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.backend.fetch_block(request)
    }
}
