pub mod coord_bounds;
pub mod geotiff;
pub mod http;
pub mod icechunk;
pub mod netcdf;
pub mod procedural;
pub mod zarr;

pub use coord_bounds::{
    fetch_all_dimension_coordinates, fetch_all_dimension_coordinates_for_variables,
    get_cached_coord_bounds, get_cached_coord_bounds_scoped, get_cached_coord_bounds_with_rank,
    read_coord_bounds, read_coord_bounds_scoped, read_coord_bounds_with_rank,
};
pub use geotiff::GeoTiffBlockStore;
pub use icechunk::IcechunkBlockStore;
pub use netcdf::NetCdfBlockStore;
pub use procedural::ProceduralBlockStore;
pub use zarr::{GenericZarrBlockStore, WasmZarrBlockStore, ZarrBlockStore};

/// Lets go of what the browser's stores keep for the dataset at `uri`, once
/// it is closed: metadata, headers, downloaded chunks and manifests.
pub fn release_stores(uri: &str) {
    WasmZarrBlockStore::release(uri);
    icechunk::wasm::WasmIcechunkBlockStore::release(uri);
    geotiff::WasmGeoTiffBlockStore::release(uri);
}
