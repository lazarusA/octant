use crate::utils::executor::{TokioBlockOn, get_shared_tokio_rt};
use object_store::ClientOptions;
use object_store::http::HttpBuilder;
use std::error::Error;
use std::sync::Arc;
use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncStorageAdapter;
use zarrs_object_store::AsyncObjectStore;

/// Helper function to build a synchronous Zarr storage adapter over HTTP object_store.
pub fn build_sync_store(url: &str) -> Result<ReadableWritableListableStorage, Box<dyn Error>> {
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
