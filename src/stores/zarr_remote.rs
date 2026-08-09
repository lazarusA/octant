use super::{DataStore, DatasetMetadata};
use crate::utils as zarr_utils;
use std::error::Error;

pub struct ZarrRemoteStore {
    pub store_url: String,
}

impl ZarrRemoteStore {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            store_url: url.into(),
        }
    }
}

impl DataStore for ZarrRemoteStore {
    fn store_type(&self) -> &'static str {
        "Remote Zarr (V2 & V3 / object_store)"
    }

    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>> {
        let base_url = self.store_url.trim_end_matches('/');
        let store = zarr_utils::build_sync_store(base_url).map_err(|e| e as Box<dyn Error>)?;
        let variables = zarr_utils::extract_store_variables_consolidated(store, base_url)?;
        let dataset_name = base_url
            .split('/')
            .next_back()
            .unwrap_or("remote.zarr")
            .to_string();

        Ok(DatasetMetadata {
            name: dataset_name,
            store_type: self.store_type().to_string(),
            variables,
            dimension_coordinates: std::collections::HashMap::new(),
        })
    }
}
