//! WebAssembly streaming and loader module for GeoTIFF and COG datasets.

pub mod inspect;
pub mod store;

pub use inspect::inspect_wasm_remote_geotiff;
pub use store::WasmGeoTiffBlockStore;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod open_tests;
