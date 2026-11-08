//! Local files and folders in the browser: picked (`pick.rs`) or dropped
//! (`drop.rs`), registered under a `local://` URL (`path.rs`) and read by
//! range (`read.rs`) through the same `http` functions remote stores use, so
//! they open as remote Zarr, Icechunk or GeoTIFF. Session only: a reload
//! forgets them. The desktop reads local paths directly and has none.

pub mod documents;
pub mod path;

#[cfg(test)]
mod documents_tests;

#[cfg(target_arch = "wasm32")]
mod consolidate;
#[cfg(target_arch = "wasm32")]
mod drop;
#[cfg(target_arch = "wasm32")]
mod pick;
#[cfg(target_arch = "wasm32")]
mod read;
#[cfg(target_arch = "wasm32")]
mod registry;

pub use path::{SCHEME, id_of};

#[cfg(target_arch = "wasm32")]
pub use drop::install_drop_listener;
#[cfg(target_arch = "wasm32")]
pub use pick::{pick_file, pick_folder};
#[cfg(target_arch = "wasm32")]
pub use read::{read_all, read_range, size};
#[cfg(target_arch = "wasm32")]
pub use registry::{kind_of, release, take_opened};

/// The store kind of the local source at `url`; the desktop has none.
#[cfg(not(target_arch = "wasm32"))]
pub fn kind_of(_url: &str) -> Option<crate::data::DataSourceKind> {
    None
}

/// Forgets the local source at `url`; the desktop has none.
#[cfg(not(target_arch = "wasm32"))]
pub fn release(_url: &str) {}
