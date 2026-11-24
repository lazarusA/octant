//! Shared HTTP network transport utilities for WebAssembly and desktop targets.

pub mod fetch;
#[cfg(any(target_arch = "wasm32", test))]
mod gcs;
pub mod size;

pub use fetch::{fetch_url_byte_range, fetch_url_bytes, is_missing, is_not_found};
pub use size::fetch_url_size;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use fetch::get_http_client;
