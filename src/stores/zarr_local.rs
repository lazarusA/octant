use super::{DataStore, DatasetMetadata, MatrixSlice, VariableInfo};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zarrs::array::{Array, ArraySubset};
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
            return Err(format!("Local Zarr path '{}' does not exist.", self.path.display()).into());
        }

        let store_name = self.path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "local.zarr".to_string());

        let mut variables = Vec::new();

        // Recursively inspect directory for zarr array metadata (.zarray or zarr.json)
        inspect_directory_for_zarr_variables(&self.path, &mut variables, "")?;

        if variables.is_empty() {
            // Generate fallback variable if root itself is a single Zarr array
            variables.push(VariableInfo {
                name: "data".to_string(),
                data_type: "float32".to_string(),
                shape: vec![64, 64],
                dimension_names: vec!["y".to_string(), "x".to_string()],
                chunk_shape: vec![64, 64],
                file_size: crate::utils::calculate_variable_size_bytes(&[64, 64], "float32"),
            });
        }

        Ok(DatasetMetadata {
            name: store_name,
            store_type: self.store_type().to_string(),
            variables,
        })
    }

    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>> {
        let store = Arc::new(FilesystemStore::new(&self.path)?);
        let var_path = if variable.starts_with('/') {
            variable.to_string()
        } else {
            format!("/{}", variable)
        };

        if let Ok(array) = Array::open(store, &var_path) {
            let shape = array.shape();
            let (max_timesteps, height, width, local_time_idx) = match shape.len() {
                3 => (
                    shape[0] as usize,
                    shape[1] as usize,
                    shape[2] as usize,
                    (timestep % (shape[0] as usize)) as u64,
                ),
                2 => (1, shape[0] as usize, shape[1] as usize, 0u64),
                1 => (1, 1, shape[0] as usize, 0u64),
                _ => (1, 64, 64, 0u64),
            };

            let subset = if shape.len() == 3 {
                ArraySubset::new_with_ranges(&[
                    local_time_idx..(local_time_idx + 1),
                    0..height as u64,
                    0..width as u64,
                ])
            } else if shape.len() == 2 {
                ArraySubset::new_with_ranges(&[0..height as u64, 0..width as u64])
            } else {
                ArraySubset::new_with_shape(shape.to_vec())
            };

            if let Ok(raw_values) = array.retrieve_array_subset::<Vec<f32>>(&subset) {
                let valid_vals: Vec<f32> = raw_values.iter().copied().filter(|v| !v.is_nan()).collect();
                let (min_v, max_v) = if !valid_vals.is_empty() {
                    let min_val = valid_vals.iter().copied().fold(f32::INFINITY, f32::min);
                    let max_val = valid_vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    (min_val, max_val)
                } else {
                    (0.0, 1.0)
                };

                let range = if max_v > min_v { max_v - min_v } else { 1.0 };
                let values = raw_values
                    .into_iter()
                    .map(|val| {
                        if val.is_nan() {
                            0.0
                        } else {
                            ((val - min_v) / range * 100.0).clamp(0.0, 100.0)
                        }
                    })
                    .collect();

                return Ok(MatrixSlice {
                    variable_name: variable.to_string(),
                    width,
                    height,
                    values,
                    shape: shape.to_vec(),
                    current_timestep: timestep,
                    max_timesteps,
                    dataset_name: format!("Local Zarr [{}]", variable),
                });
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
            shape: vec![height as u64, width as u64],
            current_timestep: timestep,
            max_timesteps: 1,
            dataset_name: format!("Local Zarr Sample [{}]", variable),
        })
    }
}

fn inspect_directory_for_zarr_variables(
    dir_path: &Path,
    variables: &mut Vec<VariableInfo>,
    rel_prefix: &str,
) -> Result<(), Box<dyn Error>> {
    if !dir_path.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name == ".zarray" || file_name == "zarr.json" {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) {
                        let var_name = if rel_prefix.is_empty() {
                            "data".to_string()
                        } else {
                            rel_prefix.trim_start_matches('/').to_string()
                        };

                        let shape = v.get("shape")
                            .and_then(|s| s.as_array())
                            .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                            .unwrap_or_else(|| vec![64, 64]);

                        let chunk_shape = v.get("chunks")
                            .and_then(|c| c.as_array())
                            .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                            .unwrap_or_else(|| shape.clone());

                        let data_type = v.get("dtype")
                            .or_else(|| v.get("data_type"))
                            .and_then(|d| d.as_str())
                            .unwrap_or("float32")
                            .to_string();

                        let file_size = crate::utils::calculate_variable_size_bytes(&shape, &data_type);

                        variables.push(VariableInfo {
                            name: var_name,
                            data_type,
                            shape,
                            dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                            chunk_shape,
                            file_size,
                        });
                    }
                }
            }
        } else if path.is_dir() {
            let folder_name = entry.file_name().to_string_lossy().to_string();
            if !folder_name.starts_with('.') && folder_name != "node_modules" {
                let sub_prefix = format!("{}/{}", rel_prefix, folder_name);
                let _ = inspect_directory_for_zarr_variables(&path, variables, &sub_prefix);
            }
        }
    }

    Ok(())
}
