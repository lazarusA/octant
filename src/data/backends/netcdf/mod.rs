//! NetCDF backend implementation for `BlockStore`.
//!
//! The desktop reads classic NetCDF, 64-bit offset, CDF-5 and NetCDF-4/HDF5 files through
//! netCDF-C (`c_reader.rs`); the browser reads NetCDF-4/HDF5 in its data workers and both
//! targets read MATLAB v7.3 `.mat` files through `hdf5-pure` (`pure/`). Everything above
//! the reader (`reader.rs`) is shared.

#[cfg(not(target_arch = "wasm32"))]
pub mod attrs;
#[cfg(not(target_arch = "wasm32"))]
mod c_reader;
pub mod coord_read;
pub mod coord_scan;
pub mod coords;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
pub mod inspect;
pub mod pure;
pub mod reader;
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
