//! Zarr backend implementations and storage abstractions.

pub mod block;
mod block_coords;
#[cfg(test)]
mod block_coords_tests;
pub mod coords;
#[cfg(test)]
mod coords_numeric_tests;
#[cfg(test)]
mod coords_tests;
pub mod generic;
pub mod slice;
pub mod storage;
pub mod store;
pub mod wasm;
pub mod zstd_shim;

pub use block::{fetch_block_from_cached_array, fetch_block_with_progress};
pub use coords::read_coordinate;
pub use generic::GenericZarrBlockStore;
pub use slice::{retrieve_array_subset_as_f32, retrieve_array_subset_as_f64};
pub use storage::{build_sync_store, open_local_storage};
pub use store::ZarrBlockStore;
pub use wasm::{WasmZarrBlockStore, inspect_wasm_remote_zarr, load_one_wasm_with_progress};
