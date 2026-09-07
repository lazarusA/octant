//! Synchronous Zarr storage construction.

use std::error::Error;
use std::sync::Arc;
use zarrs::storage::ReadableWritableListableStorage;

#[cfg(not(target_arch = "wasm32"))]
use crate::utils::executor::{TokioBlockOn, get_shared_tokio_rt};
#[cfg(not(target_arch = "wasm32"))]
use object_store::ClientOptions;
#[cfg(not(target_arch = "wasm32"))]
use object_store::http::HttpBuilder;
#[cfg(not(target_arch = "wasm32"))]
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncStorageAdapter;
#[cfg(not(target_arch = "wasm32"))]
use zarrs_object_store::AsyncObjectStore;

#[cfg(not(target_arch = "wasm32"))]
/// Builds a synchronous Zarr storage adapter over HTTP object_store, for
/// remote sources.
pub fn build_sync_store(
    url: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let clean_url = url.trim_end_matches('/');

    let options = ClientOptions::new()
        .with_allow_http(true)
        .with_allow_invalid_certificates(true)
        .with_timeout(std::time::Duration::from_secs(30))
        .with_connect_timeout(std::time::Duration::from_secs(10));

    let http_store = HttpBuilder::new()
        .with_url(clean_url)
        .with_client_options(options)
        .build()?;

    let async_store = Arc::new(AsyncObjectStore::new(http_store));

    let rt = get_shared_tokio_rt();

    let sync_store: ReadableWritableListableStorage = Arc::new(AsyncToSyncStorageAdapter::new(
        async_store,
        TokioBlockOn(rt.clone()),
    ));

    Ok(sync_store)
}

#[cfg(not(target_arch = "wasm32"))]
/// Builds a storage handle for a local Zarr store rooted at `path`.
pub fn open_local_storage(
    path: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let clean_path = path.strip_prefix("file://").unwrap_or(path);
    let expanded = crate::utils::expand_tilde(clean_path);
    let dir_path = if expanded.is_file() {
        expanded
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(expanded)
    } else {
        expanded
    };
    let store = zarrs::filesystem::FilesystemStore::new(dir_path)?;

    Ok(Arc::new(store))
}

#[cfg(target_arch = "wasm32")]
/// Builds a synchronous Zarr storage handle for remote sources on WASM.
pub fn build_sync_store(
    _url: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let store = Arc::new(zarrs::storage::store::MemoryStore::new());
    Ok(store)
}

#[cfg(target_arch = "wasm32")]
/// Builds a storage handle for a local Zarr store on WASM.
pub fn open_local_storage(
    _path: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    Err("Local file system is only available on desktop. Please provide a remote HTTP URL or choose a catalog dataset.".into())
}
