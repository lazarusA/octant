use super::{DataStore, DatasetMetadata, MatrixSlice};
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

    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>> {
        let (width, height) = (64, 64);
        let (raw_data, min_val, max_val) =
            crate::data::procedural::generate_procedural_matrix(width, height, timestep);

        Ok(MatrixSlice {
            variable_name: variable.to_string(),
            width,
            height,
            values: raw_data,
            min_val,
            max_val,
            shape: vec![height as u64, width as u64],
            current_timestep: timestep,
            max_timesteps: 1,
            dataset_name: format!("Remote Zarr [{}]", variable),
        })
    }

    fn fetch_slice_range(
        &self,
        variable: &str,
        start_step: usize,
        count: usize,
    ) -> Result<Vec<MatrixSlice>, Box<dyn Error>> {
        let mut fallback = Vec::with_capacity(count);
        for i in 0..count {
            fallback.push(self.fetch_slice(variable, start_step + i)?);
        }
        Ok(fallback)
    }
}
