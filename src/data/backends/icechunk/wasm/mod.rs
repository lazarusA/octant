//! Lightweight pure-Rust Icechunk snapshot and manifest resolver for WebAssembly.
//!
//! Provides read-only streaming of Icechunk datasets directly in web browsers without
//! Tokio, MIO, or native socket dependencies.

#[cfg(target_arch = "wasm32")]
mod chunks;
pub mod discovery;
pub mod header;
pub mod inspect;
pub mod loader;
pub mod manifests;
pub mod preload;
pub mod store;
#[cfg(test)]
mod tests;

pub use header::{decompress_icechunk_file, decompress_payload, try_decompress_zstd};
pub use inspect::inspect_wasm_remote_icechunk;
pub use loader::load_one_icechunk_wasm_with_progress;
pub use store::{ArrayManifestInfo, WasmIcechunkBlockStore};

// Backwards compatibility re-export
pub use crate::utils::remote::s3_to_https;
