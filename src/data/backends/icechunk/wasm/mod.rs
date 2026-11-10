//! Lightweight pure-Rust Icechunk snapshot and manifest resolver for WebAssembly.
//!
//! Provides read-only streaming of Icechunk datasets directly in web browsers without
//! Tokio, MIO, or native socket dependencies.

#[cfg(target_arch = "wasm32")]
mod chunks;
pub mod discovery;
pub mod header;
#[cfg(target_arch = "wasm32")]
pub mod inspect;
pub mod manifest_cache;
pub mod manifests;
#[cfg(target_arch = "wasm32")]
pub mod preload;
#[cfg(target_arch = "wasm32")]
mod shards;
#[cfg(target_arch = "wasm32")]
pub mod store;
#[cfg(test)]
mod tests;

pub use header::decompress_icechunk_file;
#[cfg(target_arch = "wasm32")]
pub use inspect::inspect_wasm_remote_icechunk;
#[cfg(target_arch = "wasm32")]
pub use store::{ArrayManifestInfo, WasmIcechunkBlockStore};
