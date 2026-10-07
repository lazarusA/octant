//! Zarr backend implementations and storage abstractions.

pub mod block;
pub mod generic;
pub mod slice;
pub mod storage;
pub mod store;
pub mod strings;
#[cfg(test)]
mod strings_tests;
pub mod wasm;
pub mod zstd_shim;

pub use block::{fetch_block_from_cached_array, fetch_block_with_progress};
pub use generic::GenericZarrBlockStore;
pub use slice::retrieve_array_subset_as_f32;
pub use storage::{build_sync_store, open_local_storage};
pub use store::ZarrBlockStore;
pub use strings::retrieve_array_as_strings;
pub use wasm::{WasmZarrBlockStore, inspect_wasm_remote_zarr, load_one_wasm_with_progress};
