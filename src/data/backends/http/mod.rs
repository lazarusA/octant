//! Shared HTTP network transport utilities for WebAssembly and desktop targets.

pub mod fetch;

pub use fetch::{fetch_url_byte_range, fetch_url_bytes};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use fetch::get_http_client;
