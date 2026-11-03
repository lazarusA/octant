//! WebAssembly fallback stub for NetCDF backend.

use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

#[derive(Debug, Clone)]
pub struct NetCdfBlockStore;

impl NetCdfBlockStore {
    pub fn open_local(_path: &str) -> Result<Self, BlockStoreError> {
        Err("NetCDF storage backend is not supported on WebAssembly targets".into())
    }

    pub fn file_path(&self) -> &str {
        ""
    }
}

impl BlockStore for NetCdfBlockStore {
    fn backend_name(&self) -> &str {
        "NetCDF (unsupported on WASM)"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Err("NetCDF is not supported on WASM".into())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        Err("NetCDF is not supported on WASM".into())
    }

    fn fetch_block_with_progress(
        &self,
        _request: &SliceRequest,
        _on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        Err("NetCDF is not supported on WASM".into())
    }
}
