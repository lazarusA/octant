//! Constructs the appropriate backend for a DataSource.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use super::{
    block_store::BlockStoreError,
    data_source::DataSource,
    store_handle::StoreHandle,
};

#[cfg(not(target_arch = "wasm32"))]
use super::{
    backends::{icechunk::IcechunkBlockStore, zarr::ZarrBlockStore},
    block_store::BlockStore,
    data_source::DataSourceKind,
};

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
    pub fn open(_source: DataSource) -> Result<StoreHandle, BlockStoreError> {
        Err("Native Zarr / Icechunk stores are not supported in WebAssembly mode".into())
    }
}
