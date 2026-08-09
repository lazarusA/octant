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

pub trait DataStore: Send + Sync {
    fn store_type(&self) -> &'static str;
    fn inspect(&self) -> Result<DatasetMetadata, Box<dyn Error>>;
}
