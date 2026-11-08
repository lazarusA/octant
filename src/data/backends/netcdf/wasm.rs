//! WebAssembly stub for the NetCDF backend: nothing reads NetCDF in the browser yet.

use super::reader::NcReader;
use crate::data::blocks::BlockStoreError;

#[derive(Debug, Clone)]
pub struct NetCdfBlockStore {
    uri: String,
}

impl NetCdfBlockStore {
    pub fn open_local(_uri: &str) -> Result<Self, BlockStoreError> {
        Err("NetCDF storage backend is not supported on WebAssembly targets".into())
    }

    pub fn file_path(&self) -> &str {
        &self.uri
    }

    pub(super) fn with_reader<R>(
        &self,
        _f: impl FnOnce(&dyn NcReader) -> Result<R, BlockStoreError>,
    ) -> Result<R, BlockStoreError> {
        Err("NetCDF is not supported on WASM".into())
    }
}
