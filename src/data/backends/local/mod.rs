//! Local files and folders in the browser: picked (`pick.rs`) or dropped
//! (`drop.rs`), registered under a `local://` URL (`path.rs`) and read by
//! range (`read.rs`) through the same `http` functions remote stores use, so
//! they open as remote Zarr, Icechunk or GeoTIFF. Session only: a reload
//! forgets them. The desktop reads local paths directly and has none.
//! Only the page holds the files: a data worker asks the page for what it
//! reads (`data::worker`), as dropped folders cannot be sent to a worker.

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

/// The bytes of the file at local `url` (`read.rs`).
#[cfg(target_arch = "wasm32")]
pub async fn read_all(url: &str) -> Result<Vec<u8>, String> {
    use crate::data::worker::{self, LocalRead};
    if worker::in_worker() {
        return worker::read_from_page(url, LocalRead::All).await;
    }
    read::read_all(url).await
}

/// `length` bytes of the file at local `url` from `offset` (`read.rs`).
#[cfg(target_arch = "wasm32")]
pub async fn read_range(url: &str, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    use crate::data::worker::{self, LocalRead};
    if worker::in_worker() {
        let read = LocalRead::Range { offset, length };
        return worker::read_from_page(url, read).await;
    }
    read::read_range(url, offset, length).await
}

/// The `File` of the local file source at `url`, which a data worker reads itself.
#[cfg(target_arch = "wasm32")]
pub fn file(url: &str) -> Result<web_sys::File, String> {
    registry::file_of(url).ok_or_else(|| format!("'{url}' is not one picked or dropped file"))
}

/// The size in bytes of the file at local `url` (`read.rs`).
#[cfg(target_arch = "wasm32")]
pub async fn size(url: &str) -> Result<u64, String> {
    use crate::data::worker::{self, LocalRead};
    if worker::in_worker() {
        let bytes = worker::read_from_page(url, LocalRead::Size).await?;
        let size: [u8; 8] = bytes
            .try_into()
            .map_err(|_| "Bad size answer".to_string())?;
        return Ok(u64::from_le_bytes(size));
    }
    read::size(url).await
}
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
