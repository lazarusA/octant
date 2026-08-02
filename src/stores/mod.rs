pub mod zarr_local;
pub mod zarr_remote;
pub mod icechunk_local;
pub mod icechunk_remote;

use std::error::Error;

#[derive(Debug, Clone)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub data_type: String,
    pub shape: Vec<u64>,
    pub dimension_names: Vec<String>,
    pub chunk_shape: Vec<u64>,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatrixSlice {
    pub variable_name: String,
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub shape: Vec<u64>,
    pub current_timestep: usize,
    pub max_timesteps: usize,
    pub dataset_name: String,
}

pub trait DataStore: Send + Sync {
    fn store_type(&self) -> &'static str;
    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>>;
    fn fetch_slice(&self, variable: &str, timestep: usize) -> Result<MatrixSlice, Box<dyn Error>>;
}
