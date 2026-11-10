//! Icechunk backend implementation for `BlockStore`.

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{IcechunkBlockStore, build_sync_icechunk_store, release};
