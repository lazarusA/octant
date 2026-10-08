//! TIFF and Cloud-Optimized GeoTIFF (COG) storage backend.

pub mod bands;
pub mod blit;
pub mod coords;
pub mod decode;
mod ifd_check;
pub mod inspect;
pub mod palette;
pub mod reader;
pub mod slicing;
pub mod store;
pub mod wasm;

#[cfg(test)]
mod axes_tests;
#[cfg(test)]
mod bands_tests;
#[cfg(test)]
mod ifd_check_tests;
#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub mod tests;

pub use inspect::inspect_tiff;
pub use store::GeoTiffBlockStore;
pub use wasm::{
    WasmGeoTiffBlockStore, inspect_wasm_remote_geotiff, load_one_geotiff_wasm_with_progress,
};
