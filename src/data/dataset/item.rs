//! A dataset is a selected source plus its open StoreHandle.
//!
//! Multiple Dataset instances can coexist, allowing variables from
//! completely different sources to be displayed together.

use super::{handle::StoreHandle, source::DataSource};
use crate::data::metadata::DatasetMetadata;

#[derive(Clone)]
pub struct Dataset {
    pub id: String,
    pub source: DataSource,
    pub store: StoreHandle,
    pub metadata: Option<DatasetMetadata>,
}

impl Dataset {
    pub fn new(id: impl Into<String>, source: DataSource, store: StoreHandle) -> Self {
        Self {
            id: id.into(),
            source,
            store,
            metadata: None,
        }
    }
}
