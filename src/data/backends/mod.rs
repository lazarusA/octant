pub mod coord_bounds;
pub mod geotiff;
pub mod http;
pub mod icechunk;
pub mod local;
pub mod netcdf;
pub mod procedural;
pub mod web;
pub mod zarr;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod release_tests;

pub use coord_bounds::fetch_all_dimension_coordinates_for_variables;
pub use geotiff::GeoTiffBlockStore;
#[cfg(not(target_arch = "wasm32"))]
pub use icechunk::IcechunkBlockStore;
pub use netcdf::NetCdfBlockStore;
pub use procedural::ProceduralBlockStore;
pub use zarr::{GenericZarrBlockStore, WasmZarrBlockStore, ZarrBlockStore};

/// Lets go of what the stores keep for the dataset at `uri`, once it is closed:
/// metadata, headers, downloaded chunks and manifests, the data workers' too
/// (`data::worker`), and on the desktop its opened TIFF and Icechunk session, so
/// opening it again reads the file or repository anew.
pub fn release_stores(uri: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        geotiff::GeoTiffBlockStore::release(uri);
        icechunk::release(uri);
    }
    WasmZarrBlockStore::release(uri);
    #[cfg(target_arch = "wasm32")]
    icechunk::wasm::WasmIcechunkBlockStore::release(uri);
    geotiff::WasmGeoTiffBlockStore::release(uri);
    #[cfg(target_arch = "wasm32")]
    netcdf::wasm::release(uri);
    #[cfg(target_arch = "wasm32")]
    if !crate::data::worker::in_worker() {
        crate::data::worker::release(uri);
    }
}
