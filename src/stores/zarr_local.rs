use super::{DataStore, DatasetMetadata, MatrixSlice};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zarrs::filesystem::FilesystemStore;

pub struct ZarrLocalStore {
    pub path: PathBuf,
}

impl ZarrLocalStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl DataStore for ZarrLocalStore {
    fn store_type(&self) -> &'static str {
        "Local Zarr"
    }

    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>> {
        if !self.path.exists() {
            return Err(
                format!("Local Zarr path '{}' does not exist.", self.path.display()).into(),
            );
        }

        let store_name = self
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "local.zarr".to_string());

        let store_path_str = self.path.to_string_lossy().to_string();
        let (variables, dimension_coordinates) = if let Ok(store) = FilesystemStore::new(&self.path)
        {
            let store_arc = Arc::new(store);
            let vars = crate::utils::extract_store_variables_consolidated(
                store_arc.clone(),
                &store_path_str,
            )?;
            let dim_names: Vec<String> = vars
                .iter()
                .flat_map(|v| v.dimension_names.clone())
                .collect();
            let coords = crate::utils::fetch_all_dimension_coordinates(
                store_arc,
                &dim_names,
                Some(&store_path_str),
            );
            (vars, coords)
        } else {
            (Vec::new(), std::collections::HashMap::new())
        };

        Ok(DatasetMetadata {
            name: store_name,
            store_type: self.store_type().to_string(),
            variables,
            dimension_coordinates,
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
            dataset_name: format!("Local Zarr [{}]", variable),
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
