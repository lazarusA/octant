pub mod coord_bounds;
pub mod geotiff;
pub mod http;
pub mod icechunk;
pub mod local;
pub mod netcdf;
pub mod procedural;
pub mod zarr;

pub use coord_bounds::fetch_all_dimension_coordinates_for_variables;
pub use geotiff::GeoTiffBlockStore;
pub use icechunk::IcechunkBlockStore;
pub use netcdf::NetCdfBlockStore;
pub use procedural::ProceduralBlockStore;
pub use zarr::{GenericZarrBlockStore, WasmZarrBlockStore, ZarrBlockStore};

/// Lets go of what the browser's stores keep for the dataset at `uri`, once
/// it is closed: metadata, headers, downloaded chunks and manifests, the
/// data workers' too (`data::worker`).
pub fn release_stores(uri: &str) {
    WasmZarrBlockStore::release(uri);
    icechunk::wasm::WasmIcechunkBlockStore::release(uri);
    geotiff::WasmGeoTiffBlockStore::release(uri);
    #[cfg(target_arch = "wasm32")]
    if !crate::data::worker::in_worker() {
        crate::data::worker::release(uri);
    }
}

/// Reads the metadata of the browser store `source` into its store registry,
/// as both the page (opening a dataset) and a data worker (opening the store
/// for its loads) do; `None` for stores whose `StoreHandle::inspect` reads it.
#[cfg(target_arch = "wasm32")]
pub async fn inspect_web(
    source: &crate::data::DataSource,
) -> Option<Result<crate::data::DatasetMetadata, String>> {
    use crate::data::DataSourceKind as Kind;
    let uri = &source.uri;
    Some(match source.kind {
        Kind::RemoteZarr => zarr::inspect_wasm_remote_zarr(uri).await,
        Kind::RemoteIcechunk => icechunk::wasm::inspect_wasm_remote_icechunk(uri).await,
        Kind::RemoteGeoTiff => geotiff::wasm::inspect_wasm_remote_geotiff(uri).await,
        _ => return None,
    })
}
