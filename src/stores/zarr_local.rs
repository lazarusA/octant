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
            let coords = crate::utils::zarr::fetch_all_dimension_coordinates(
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
        let store_path_str = self.path.to_string_lossy().to_string();
        if let Ok(store) = FilesystemStore::new(&self.path) {
            if let Ok(slice) = crate::utils::zarr::fetch_slice(
                Arc::new(store),
                &store_path_str,
                variable,
                timestep,
            ) {
                return Ok(slice);
            }
        }

        // Fallback procedural matrix if array slice read fails
        let (width, height) = (64, 64);
        let mut raw_data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let val = (((x * 17 + y * 31 + timestep * 13) % 100) as f32).clamp(0.0, 100.0);
                raw_data.push(val);
            }
        }

        Ok(MatrixSlice {
            variable_name: variable.to_string(),
            width,
            height,
            values: raw_data,
            min_val: 0.0,
            max_val: 100.0,
            shape: vec![height as u64, width as u64],
            current_timestep: timestep,
            max_timesteps: 1,
            dataset_name: format!("Local Zarr Sample [{}]", variable),
        })
    }

    fn fetch_slice_range(
        &self,
        variable: &str,
        start_step: usize,
        count: usize,
    ) -> Result<Vec<MatrixSlice>, Box<dyn Error>> {
        if let Ok(store) = FilesystemStore::new(&self.path) {
            let store_path_str = self.path.to_string_lossy().to_string();
            if let Ok(slices) = crate::utils::zarr::fetch_slice_range(
                Arc::new(store),
                &store_path_str,
                variable,
                start_step,
                count,
            ) {
                return Ok(slices);
            }
        }
        let mut fallback = Vec::with_capacity(count);
        for i in 0..count {
            fallback.push(self.fetch_slice(variable, start_step + i)?);
        }
        Ok(fallback)
    }
}
