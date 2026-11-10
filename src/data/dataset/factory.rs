//! Constructs the appropriate backend for a DataSource.

use std::sync::Arc;

use super::{
    handle::StoreHandle,
    source::{DataSource, DataSourceKind},
};
#[cfg(target_arch = "wasm32")]
use crate::data::backends::web::WebFormat;
use crate::data::blocks::{BlockStore, BlockStoreError};

#[cfg(not(target_arch = "wasm32"))]
use crate::data::backends::{
    geotiff::GeoTiffBlockStore, icechunk::IcechunkBlockStore, zarr::ZarrBlockStore,
};
use crate::data::backends::{netcdf::NetCdfBlockStore, procedural::ProceduralBlockStore};

pub struct SourceFactory;

impl SourceFactory {
    pub fn open(source: DataSource) -> Result<StoreHandle, BlockStoreError> {
        let backend: Arc<dyn BlockStore> = match &source.kind {
            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::LocalZarr => Arc::new(ZarrBlockStore::open_local(&source.uri)?),
            // A desktop path, which the browser cannot read: its store says so.
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::LocalZarr
            | DataSourceKind::LocalIcechunk
            | DataSourceKind::LocalGeoTiff => {
                crate::data::backends::WasmZarrBlockStore::open_local_notice(&source.uri)
            }

            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::RemoteZarr => Arc::new(ZarrBlockStore::open_remote(&source.uri)?),
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::RemoteZarr if WebFormat::of(&source) == WebFormat::Zarr => {
                crate::data::backends::WasmZarrBlockStore::get_or_create(&source.uri)
            }

            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::LocalIcechunk | DataSourceKind::RemoteIcechunk => {
                Arc::new(IcechunkBlockStore::open(&source.uri)?)
            }
            // `icechunk+` addresses opened as Zarr read as Icechunk (`WebFormat::of`).
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::RemoteZarr | DataSourceKind::RemoteIcechunk => {
                crate::data::backends::icechunk::wasm::WasmIcechunkBlockStore::get_or_create(
                    &source.uri,
                )
            }

            DataSourceKind::Procedural => Arc::new(ProceduralBlockStore::open(&source.uri)?),

            DataSourceKind::NetCdf => Arc::new(NetCdfBlockStore::open_local(&source.uri)?),

            #[cfg(not(target_arch = "wasm32"))]
            DataSourceKind::LocalGeoTiff | DataSourceKind::RemoteGeoTiff => {
                Arc::new(GeoTiffBlockStore::open(&source.uri)?)
            }
            #[cfg(target_arch = "wasm32")]
            DataSourceKind::RemoteGeoTiff => {
                crate::data::backends::geotiff::WasmGeoTiffBlockStore::get_or_create(&source.uri)
            }
        };

        Ok(StoreHandle::new(source, backend))
    }
}
