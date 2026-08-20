//! Synchronous Zarr storage construction.

use std::error::Error;
use std::sync::Arc;

use object_store::ClientOptions;
use object_store::http::HttpBuilder;

use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncStorageAdapter;
use zarrs_object_store::AsyncObjectStore;

use crate::utils::executor::{TokioBlockOn, get_shared_tokio_rt};

/// Builds a synchronous Zarr storage adapter over HTTP object_store, for
/// remote sources.
pub fn build_sync_store(
    url: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let clean_url = url.trim_end_matches('/');

    let options = ClientOptions::new()
        .with_allow_http(true)
        .with_allow_invalid_certificates(true);

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

/// Helper to expand tilde `~` or `~/` in path strings to the user's home directory.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            return std::path::PathBuf::from(home).join(stripped);
        }
    } else if path == "~"
        && let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
    {
        return std::path::PathBuf::from(home);
    }
    std::path::PathBuf::from(path)
}

/// Builds a storage handle for a local Zarr store rooted at `path`.
pub fn open_local_storage(
    path: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let clean_path = path.strip_prefix("file://").unwrap_or(path);
    let expanded = expand_tilde(clean_path);
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
