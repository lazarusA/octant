use super::{DataStore, DatasetMetadata, MatrixSlice};
use std::error::Error;

pub struct IcechunkRemoteStore {
    pub endpoint_url: String,
}

impl IcechunkRemoteStore {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            endpoint_url: url.into(),
        }
    }
}

impl DataStore for IcechunkRemoteStore {
    fn store_type(&self) -> &'static str {
        "Icechunk Store (main branch)"
    }

    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>> {
        let base_url = self.endpoint_url.trim_end_matches('/');
        let store = crate::utils::icechunk::build_sync_icechunk_store(base_url)?;
        let variables =
            crate::utils::extract_store_variables_consolidated(store.clone(), base_url)?;

        let dim_names: Vec<String> = variables
            .iter()
            .flat_map(|v| v.dimension_names.clone())
            .collect();
        let dimension_coordinates =
            crate::utils::fetch_all_dimension_coordinates(store, &dim_names, Some(base_url));

        let dataset_name = base_url
            .split('/')
            .next_back()
            .unwrap_or("icechunk_store")
            .to_string();

        Ok(DatasetMetadata {
            name: dataset_name,
            store_type: self.store_type().to_string(),
            variables,
            dimension_coordinates,
        })
    }

    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>> {
        let base_url = self.endpoint_url.trim_end_matches('/');
        let store = crate::utils::icechunk::build_sync_icechunk_store(base_url)?;
        crate::utils::zarr::fetch_slice(store, base_url, variable, timestep)
    }

    fn fetch_slice_range(
        &self,
        variable: &str,
        start_step: usize,
        count: usize,
    ) -> Result<Vec<MatrixSlice>, Box<dyn Error>> {
        let base_url = self.endpoint_url.trim_end_matches('/');
        let store = crate::utils::icechunk::build_sync_icechunk_store(base_url)?;
        crate::utils::zarr::fetch_slice_range(store, base_url, variable, start_step, count)
    }
}
