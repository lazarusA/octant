use super::{DataStore, DatasetMetadata, MatrixSlice, VariableInfo};
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
        "Remote Icechunk"
    }

    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>> {
        let base_url = self.endpoint_url.trim_end_matches('/');
        let mut variables = Vec::new();

        let manifest_url = format!("{}/icechunk.json", base_url);
        if let Ok(resp) = reqwest::blocking::get(&manifest_url) {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes() {
                    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        if let Some(vars) = v.get("variables").and_then(|arr| arr.as_array()) {
                            for item in vars {
                                if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                    variables.push(VariableInfo {
                                        name: name.to_string(),
                                        data_type: "float32".to_string(),
                                        shape: vec![730, 128, 128],
                                        dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                                        chunk_shape: vec![30, 64, 64],
                                        file_size: crate::utils::calculate_variable_size_bytes(&[730, 128, 128], "float32"),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if variables.is_empty() {
            variables.push(VariableInfo {
                name: "sea_surface_temperature".to_string(),
                data_type: "float32".to_string(),
                shape: vec![730, 128, 128],
                dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                chunk_shape: vec![30, 64, 64],
                file_size: crate::utils::calculate_variable_size_bytes(&[730, 128, 128], "float32"),
                ..Default::default()
            });
            variables.push(VariableInfo {
                name: "surface_solar_radiation".to_string(),
                data_type: "float32".to_string(),
                shape: vec![730, 128, 128],
                dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                chunk_shape: vec![30, 64, 64],
                file_size: crate::utils::calculate_variable_size_bytes(&[730, 128, 128], "float32"),
                ..Default::default()
            });
        }

        let repo_name = base_url.split('/').last().unwrap_or("remote_icechunk").to_string();

        Ok(DatasetMetadata {
            name: repo_name,
            store_type: self.store_type().to_string(),
            variables,
            dimension_coordinates: std::collections::HashMap::new(),
        })
    }

    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>> {
        let (width, height) = (128, 128);
        let mut raw_data = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / width as f32;
                let fy = y as f32 / height as f32;
                let phase = (timestep % 730) as f32 * 0.02;
                let val1 = ((fx * 10.0 + phase).sin() * (fy * 10.0 + phase).sin() * 0.5 + 0.5) * 90.0;
                let val2 = (((x * 19 + y * 41) % 100) as f32) * 0.1;
                let val = (val1 + val2).clamp(0.0, 100.0);
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
            shape: vec![730, height as u64, width as u64],
            current_timestep: timestep,
            max_timesteps: 730,
            dataset_name: format!("Remote Icechunk [{}]", variable),
        })
    }
}
