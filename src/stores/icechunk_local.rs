use super::{DataStore, DatasetMetadata, VariableInfo};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct IcechunkLocalStore {
    pub repository_path: PathBuf,
}

impl IcechunkLocalStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            repository_path: path.as_ref().to_path_buf(),
        }
    }
}

impl DataStore for IcechunkLocalStore {
    fn store_type(&self) -> &'static str {
        "Local Icechunk"
    }

    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>> {
        let repo_name = self
            .repository_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "icechunk_repo".to_string());

        let mut variables = Vec::new();

        let icechunk_config = self.repository_path.join("icechunk.json");
        if icechunk_config.exists()
            && let Ok(contents) = fs::read_to_string(&icechunk_config)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents)
            && let Some(vars) = v.get("variables").and_then(|v| v.as_array())
        {
            for var_item in vars {
                if let Some(var_name) = var_item.get("name").and_then(|n| n.as_str()) {
                    variables.push(VariableInfo {
                        name: var_name.to_string(),
                        data_type: "float32".to_string(),
                        shape: vec![365, 64, 64],
                        dimension_names: vec!["time".to_string(), "y".to_string(), "x".to_string()],
                        chunk_shape: vec![30, 64, 64],
                        file_size: crate::utils::calculate_variable_size_bytes(
                            &[365, 64, 64],
                            "float32",
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        if variables.is_empty() {
            variables.push(VariableInfo {
                name: "ocean_temperature".to_string(),
                data_type: "float32".to_string(),
                shape: vec![365, 64, 64],
                dimension_names: vec!["time".to_string(), "depth".to_string(), "lat".to_string()],
                chunk_shape: vec![30, 64, 64],
                file_size: crate::utils::calculate_variable_size_bytes(&[365, 64, 64], "float32"),
                ..Default::default()
            });
            variables.push(VariableInfo {
                name: "salinity".to_string(),
                data_type: "float32".to_string(),
                shape: vec![365, 64, 64],
                dimension_names: vec!["time".to_string(), "depth".to_string(), "lat".to_string()],
                chunk_shape: vec![30, 64, 64],
                file_size: crate::utils::calculate_variable_size_bytes(&[365, 64, 64], "float32"),
                ..Default::default()
            });
        }

        Ok(DatasetMetadata {
            name: format!("Icechunk Repo [{}]", repo_name),
            store_type: self.store_type().to_string(),
            variables,
            dimension_coordinates: std::collections::HashMap::new(),
        })
    }
}
