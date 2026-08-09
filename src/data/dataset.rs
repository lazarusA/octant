//! A dataset is a selected source plus its open StoreHandle.
//!
//! Multiple Dataset instances can coexist, allowing variables from
//! completely different sources to be displayed together.

use super::{
    block_request::BlockRequest,
    data_source::DataSource,
    slice_request::{DimensionSelection, SliceRequest},
    store_handle::StoreHandle,
};

#[derive(Clone)]
pub struct Dataset {
    pub id: String,
    pub source: DataSource,
    pub store: StoreHandle,
    pub metadata: Option<crate::data::DatasetMetadata>,
    pub selected_variables: Vec<String>,
}

impl Dataset {
    pub fn new(id: impl Into<String>, source: DataSource, store: StoreHandle) -> Self {
        Self {
            id: id.into(),
            source,
            store,
            metadata: None,
            selected_variables: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, variable: impl Into<String>) {
        let variable = variable.into();

        if !self.selected_variables.contains(&variable) {
            self.selected_variables.push(variable);
        }
    }

    pub fn remove_variable(&mut self, variable: &str) {
        self.selected_variables.retain(|v| v != variable);
    }

    pub fn has_variable(&self, variable: &str) -> bool {
        self.selected_variables.iter().any(|v| v == variable)
    }

    /// Builds a `BlockRequest` for a selection against this dataset's own
    /// store.
    pub fn request(&self, slice: SliceRequest) -> BlockRequest {
        BlockRequest::new(self.store.clone(), slice)
    }

    pub fn request_variable(
        &self,
        variable: impl Into<String>,
        selections: Vec<DimensionSelection>,
    ) -> BlockRequest {
        self.request(SliceRequest::new(variable, selections))
    }
}
