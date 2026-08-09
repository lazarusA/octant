use super::{DataStore, DatasetMetadata};
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
}
