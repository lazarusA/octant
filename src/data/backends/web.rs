//! The browser's formats: each source's format decided once (`WebFormat::of`), and each
//! operation one exhaustive `match` over it, so a new format compiles only once every
//! operation handles it.

use crate::data::{DataSource, DataSourceKind as Kind};

/// How the browser reads a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebFormat {
    Zarr,
    Icechunk,
    GeoTiff,
    /// Read only in data workers (`netcdf/wasm.rs`).
    NetCdf,
    Procedural,
    /// A desktop path, which the browser cannot read; its store says so.
    DesktopPath,
}

impl WebFormat {
    /// The format of `source`. Local files and folders register as remote kinds
    /// (`local::kind_of`), so only typed desktop paths are `Local*`.
    pub fn of(source: &DataSource) -> Self {
        match source.kind {
            Kind::RemoteZarr if source.uri.starts_with("icechunk+") => Self::Icechunk,
            Kind::RemoteZarr => Self::Zarr,
            Kind::RemoteIcechunk => Self::Icechunk,
            Kind::RemoteGeoTiff => Self::GeoTiff,
            Kind::NetCdf | Kind::Grib => Self::NetCdf,
            Kind::Procedural => Self::Procedural,
            Kind::LocalZarr | Kind::LocalIcechunk | Kind::LocalGeoTiff => Self::DesktopPath,
        }
    }

    /// Whether a data worker loads its blocks, rather than the page.
    pub fn in_worker(self) -> bool {
        self != Self::DesktopPath
    }
}

#[cfg(target_arch = "wasm32")]
mod ops {
    use std::collections::HashMap;

    use super::WebFormat;
    use crate::data::backends::geotiff::WasmGeoTiffBlockStore;
    use crate::data::backends::icechunk::wasm::{
        WasmIcechunkBlockStore, inspect_wasm_remote_icechunk,
    };
    use crate::data::backends::zarr::wasm::chunk_source::{load_block, load_variable_coordinates};
    use crate::data::backends::zarr::{WasmZarrBlockStore, inspect_wasm_remote_zarr};
    use crate::data::blocks::{BlockRequest, BlockStoreError, ProgressCallback};
    use crate::data::dataset::StoreHandle;
    use crate::data::octant_block::OctantBlock;
    use crate::data::{CoordValues, DataSource, DatasetMetadata, VariableInfo};

    impl WebFormat {
        /// Reads `source`'s metadata into its store's registry, as both the page (opening a
        /// dataset) and a data worker (opening the store for its loads) do; `None` for
        /// stores whose `StoreHandle::inspect` reads it.
        pub async fn inspect(self, source: &DataSource) -> Option<Result<DatasetMetadata, String>> {
            let uri = &source.uri;
            Some(match self {
                Self::Zarr => inspect_wasm_remote_zarr(uri).await,
                Self::Icechunk => inspect_wasm_remote_icechunk(uri).await,
                Self::GeoTiff => {
                    crate::data::backends::geotiff::wasm::inspect_wasm_remote_geotiff(uri).await
                }
                // Only workers open NetCDF (`worker::host::open`): no call to it here, so
                // `hdf5-pure` stays out of the page's `.wasm`. The page asks a worker.
                Self::NetCdf => crate::data::worker::inspect(source).await,
                Self::Procedural | Self::DesktopPath => return None,
            })
        }

        /// Loads one block: Zarr and Icechunk fetch its chunks and decode them, GeoTIFF reads
        /// its tiles by range, the others read synchronously.
        pub async fn load_block(
            self,
            request: &BlockRequest,
            on_progress: ProgressCallback<'_>,
        ) -> Result<OctantBlock, BlockStoreError> {
            let uri = &request.store.source().uri;
            match self {
                Self::Zarr => {
                    load_block(
                        &*WasmZarrBlockStore::get_or_create(uri),
                        request,
                        on_progress,
                    )
                    .await
                }
                Self::Icechunk => {
                    let store = WasmIcechunkBlockStore::get_or_create(uri);
                    load_block(&*store, request, on_progress).await
                }
                Self::GeoTiff => {
                    let store = WasmGeoTiffBlockStore::get_or_create(uri).open().await?;
                    store.fetch_async(&request.slice).await
                }
                Self::NetCdf | Self::Procedural | Self::DesktopPath => {
                    request.store.fetch(&request.slice)
                }
            }
        }

        /// The coordinates of `variable`'s dimensions: Zarr and Icechunk fetch their arrays
        /// first, the others read them from their store.
        pub async fn load_coordinates(
            self,
            store: &StoreHandle,
            variable: &VariableInfo,
        ) -> Result<HashMap<String, CoordValues>, String> {
            let uri = &store.source().uri;
            match self {
                Self::Zarr => Ok(load_variable_coordinates(
                    &*WasmZarrBlockStore::get_or_create(uri),
                    variable,
                )
                .await),
                Self::Icechunk => {
                    let source = WasmIcechunkBlockStore::get_or_create(uri);
                    Ok(load_variable_coordinates(&*source, variable).await)
                }
                Self::GeoTiff | Self::NetCdf | Self::Procedural | Self::DesktopPath => store
                    .variable_coordinates(variable)
                    .map_err(|e| e.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WebFormat;
    use crate::data::{DataSource, DataSourceKind as Kind};

    #[test]
    fn every_kind_has_one_format() {
        let of = |kind, uri: &str| WebFormat::of(&DataSource::new("id", kind, uri, "name"));
        assert_eq!(of(Kind::RemoteZarr, "https://x/a.zarr"), WebFormat::Zarr);
        assert_eq!(of(Kind::RemoteZarr, "local://1/a.zarr"), WebFormat::Zarr);
        assert_eq!(
            of(Kind::RemoteZarr, "icechunk+s3://b/r"),
            WebFormat::Icechunk
        );
        assert_eq!(
            of(Kind::RemoteIcechunk, "local://2/repo"),
            WebFormat::Icechunk
        );
        assert_eq!(
            of(Kind::RemoteGeoTiff, "local://3/a.tif"),
            WebFormat::GeoTiff
        );
        assert_eq!(of(Kind::NetCdf, "https://x/a.nc"), WebFormat::NetCdf);
        assert_eq!(of(Kind::Grib, "local://4/a.grib2"), WebFormat::NetCdf);
        assert_eq!(
            of(Kind::Procedural, "procedural://x"),
            WebFormat::Procedural
        );
        for kind in [Kind::LocalZarr, Kind::LocalIcechunk, Kind::LocalGeoTiff] {
            let format = of(kind, "/home/a");
            assert_eq!(format, WebFormat::DesktopPath);
            assert!(!format.in_worker());
        }
    }
}
