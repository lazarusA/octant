//! Shared HTTP network transport utilities for WebAssembly and desktop targets.

pub mod fetch;
pub mod tail;

pub use fetch::{fetch_url_byte_range, fetch_url_bytes, is_missing, is_not_found};
pub use tail::fetch_url_tail;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use fetch::get_http_client;
