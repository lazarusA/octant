//! Constructs the appropriate backend for a DataSource.

use std::sync::Arc;

use super::{
    block_store::{BlockStore, BlockStoreError},
    data_source::DataSource,
    store_handle::StoreHandle,
};

#[cfg(not(target_arch = "wasm32"))]
use super::{
    backends::{icechunk::IcechunkBlockStore, zarr::ZarrBlockStore},
    data_source::DataSourceKind,
};

#[cfg(target_arch = "wasm32")]
pub struct WasmBlockStore {
    source_url: String,
}

#[cfg(target_arch = "wasm32")]
impl BlockStore for WasmBlockStore {
    fn backend_name(&self) -> &str {
        "wasm"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Ok(Vec::new())
    }

    fn inspect(&self) -> Result<crate::data::DatasetMetadata, BlockStoreError> {
        Err("Inspect store using async WASM inspector".into())
    }

    fn fetch_block(&self, request: &crate::data::slice_request::SliceRequest) -> Result<crate::data::octant_block::OctantBlock, BlockStoreError> {
        let dummy_store = Arc::new(zarrs::storage::store::MemoryStore::new());
        crate::data::backends::zarr_block::fetch_block(dummy_store, &self.source_url, request)
    }
}

pub struct SourceFactory;

impl SourceFactory {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(source: DataSource) -> Result<StoreHandle, BlockStoreError> {
        let backend: Arc<dyn BlockStore> = match &source.kind {
            DataSourceKind::LocalZarr => Arc::new(ZarrBlockStore::open_local(&source.uri)?),

            DataSourceKind::RemoteZarr => Arc::new(ZarrBlockStore::open_remote(&source.uri)?),

            DataSourceKind::LocalIcechunk | DataSourceKind::RemoteIcechunk => {
                Arc::new(IcechunkBlockStore::open(&source.uri)?)
            }

            DataSourceKind::NetCdf => {
                return Err("NetCDF backend not yet implemented".into());
            }

            DataSourceKind::GeoTiff => {
                return Err("GeoTIFF backend not yet implemented".into());
            }

            DataSourceKind::Other(kind) => {
                return Err(format!("Unsupported data source kind: {kind}").into());
            }
        };

        Ok(StoreHandle::new(source, backend))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn open(source: DataSource) -> Result<StoreHandle, BlockStoreError> {
        let backend = Arc::new(WasmBlockStore {
            source_url: source.uri.clone(),
        });
        Ok(StoreHandle::new(source, backend))
    }
}
