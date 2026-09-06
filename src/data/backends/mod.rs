pub mod generic_zarr;
pub mod icechunk;
#[cfg(not(target_arch = "wasm32"))]
pub mod icechunk_storage;
pub mod netcdf;
pub mod procedural;
pub mod zarr;
pub mod zarr_block;
pub mod zarr_slice;
pub mod zarr_storage;

pub use generic_zarr::GenericZarrBlockStore;
pub use netcdf::NetCdfBlockStore;
pub use procedural::ProceduralBlockStore;
