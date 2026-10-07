//! NetCDF backend implementation for `BlockStore`.
//!
//! Supports reading classic NetCDF, 64-bit offset, CDF-5, and NetCDF-4/HDF5 files.

#[cfg(not(target_arch = "wasm32"))]
pub mod attrs;
#[cfg(not(target_arch = "wasm32"))]
pub mod coord_read;
#[cfg(not(target_arch = "wasm32"))]
pub mod coord_scan;
#[cfg(not(target_arch = "wasm32"))]
pub mod coords;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub mod inspect;
#[cfg(not(target_arch = "wasm32"))]
pub mod slice;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod coord_block_tests;
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod coord_tests;
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod test_support;
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::NetCdfBlockStore;

#[cfg(target_arch = "wasm32")]
pub use wasm::NetCdfBlockStore;
