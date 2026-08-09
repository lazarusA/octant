//! Constructs the appropriate backend for a DataSource.

use std::sync::Arc;

use super::{
    block_store::{BlockStore, BlockStoreError},
    data_source::{DataSource, DataSourceKind},
    store_handle::StoreHandle,
};

use super::backends::{icechunk::IcechunkBlockStore, zarr::ZarrBlockStore};

pub struct SourceFactory;

impl SourceFactory {
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
}
