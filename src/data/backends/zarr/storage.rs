//! Synchronous Zarr storage construction.

use std::error::Error;
use std::sync::Arc;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;
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
/// Builds a synchronous Zarr storage adapter over HTTP object_store, for remote sources.
pub fn build_sync_store(
    url: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let clean_url = &crate::utils::remote::s3_as_https(url.trim_end_matches('/'));

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
        TokioBlockOn(rt),
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

pub type ZarrArrayHandle = zarrs::array::Array<dyn zarrs::storage::ReadableStorageTraits>;

/// The root group's consolidated metadata, if it has any; the attributes
/// and Zarr format of the root and of every group it lists go into `meta`,
/// so its readers do not read them again. `Err` when the store could not be
/// read (a timeout, a server error), which is worth asking again, unlike a
/// store without a root group.
pub fn root_consolidated(
    storage: &ReadableWritableListableStorage,
    meta: &crate::data::ome::MetaCache,
) -> Result<Option<ConsolidatedMetadataMetadata>, zarrs::storage::StorageError> {
    use zarrs::group::GroupCreateError;
    use zarrs::metadata::GroupMetadata;
    use zarrs::node::NodeMetadata;
    let group = match zarrs::group::Group::open(storage.clone(), "/") {
        Ok(group) => group,
        Err(GroupCreateError::StorageError(e)) => return Err(e),
        Err(_) => return Ok(None),
    };
    let keep = |path: &str, metadata: &GroupMetadata| {
        let (attrs, format) = match metadata {
            GroupMetadata::V3(m) => (&m.attributes, 3),
            GroupMetadata::V2(m) => (&m.attributes, 2),
        };
        meta.keep_found_group(path, attrs.clone(), format);
    };
    keep("", group.metadata());
    let consolidated = group.consolidated_metadata().map(|c| c.metadata);
    for (path, node) in consolidated.iter().flatten() {
        if let NodeMetadata::Group(metadata) = node {
            keep(path, metadata);
        }
    }
    Ok(consolidated)
}

/// Opens a normalized array from `storage`, from the root's `consolidated`
/// metadata when it lists the array.
pub fn open_raw_array(
    storage: &ReadableWritableListableStorage,
    clean_name: &str,
    consolidated: Option<&ConsolidatedMetadataMetadata>,
) -> Result<ZarrArrayHandle, crate::data::blocks::BlockStoreError> {
    let var_path = format!("/{clean_name}");
    let readable: zarrs::storage::ReadableStorage = storage.clone();
    if let Some(node_meta) =
        consolidated.and_then(|m| m.get(clean_name).or_else(|| m.get(&var_path)))
        && let Some(arr) = crate::utils::metadata::instantiate_array_from_node_metadata(
            readable.clone(),
            &var_path,
            node_meta,
        )
    {
        Ok(arr)
    } else {
        crate::utils::metadata::open_or_instantiate_array_normalized(readable, &var_path)
            .map_err(|e| format!("{e}").into())
    }
}
