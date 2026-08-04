pub mod icechunk_local;
pub mod icechunk_remote;
pub mod zarr_local;
pub mod zarr_remote;

use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
    pub dimension_coordinates: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct VariableInfo {
    pub name: String,
    pub data_type: String,
    pub shape: Vec<u64>,
    pub dimension_names: Vec<String>,
    pub chunk_shape: Vec<u64>,
    pub file_size: u64,
    pub units: Option<String>,
    pub long_name: Option<String>,
    pub time_coverage_start: Option<String>,
    pub time_coverage_end: Option<String>,
    pub temporal_resolution: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatrixSlice {
    pub variable_name: String,
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
    pub shape: Vec<u64>,
    pub current_timestep: usize,
    pub max_timesteps: usize,
    pub dataset_name: String,
}

impl MatrixSlice {
    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>()
            + self.variable_name.len()
            + self.dataset_name.len()
            + std::mem::size_of::<Self>()
    }
}

pub trait DataStore: Send + Sync {
    fn store_type(&self) -> &'static str;
    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>>;
    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>>;

    fn fetch_slice_range(
        &self,
        variable: &str,
        start_step: usize,
        count: usize,
    ) -> Result<Vec<MatrixSlice>, Box<dyn Error>> {
        let mut slices = Vec::with_capacity(count);
        for i in 0..count {
            let slice = self.fetch_slice(variable, start_step + i)?;
            slices.push(slice);
        }
        Ok(slices)
    }
}
