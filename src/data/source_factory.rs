//! Constructs the appropriate backend for a DataSource.

use std::sync::Arc;

use super::{
    block_store::{BlockStore, BlockStoreError},
    data_source::{DataSource, DataSourceKind},
    store_handle::StoreHandle,
};

#[cfg(not(target_arch = "wasm32"))]
use super::backends::zarr::ZarrBlockStore;
use super::backends::{
    icechunk::IcechunkBlockStore, netcdf::NetCdfBlockStore, procedural::ProceduralBlockStore,
};

pub struct SourceFactory;

impl SourceFactory {
    pub fn open(source: DataSource) -> Result<StoreHandle, BlockStoreError> {
        let backend: Arc<dyn BlockStore> = match &source.kind {
            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::LocalZarr => Arc::new(ZarrBlockStore::open_local(&source.uri)?),
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::LocalZarr => {
                crate::data::backends::WasmZarrBlockStore::open_local_notice(&source.uri)
            }

            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::RemoteZarr => Arc::new(ZarrBlockStore::open_remote(&source.uri)?),
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::RemoteZarr => {
                crate::data::backends::WasmZarrBlockStore::get_or_create(&source.uri)
            }

            DataSourceKind::LocalIcechunk | DataSourceKind::RemoteIcechunk => {
                Arc::new(IcechunkBlockStore::open(&source.uri)?)
            }

            DataSourceKind::Procedural => Arc::new(ProceduralBlockStore::open(&source.uri)?),

            DataSourceKind::NetCdf => Arc::new(NetCdfBlockStore::open_local(&source.uri)?),

            DataSourceKind::GeoTiff => {
                return Err("GeoTIFF backend not yet implemented".into());
            }

            DataSourceKind::Other(kind) => {
                if kind.to_lowercase().contains("procedural") {
                    Arc::new(ProceduralBlockStore::open(&source.uri)?)
                } else {
                    return Err(format!("Unsupported data source kind: {kind}").into());
                }
            }
        };

        Ok(StoreHandle::new(source, backend))
    }
}
