//! Desktop Icechunk storage and `IcechunkBlockStore` implementation.

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, OnceLock, RwLock};

use crate::data::DatasetMetadata;
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::utils::executor::{TokioBlockOn, get_shared_tokio_rt};
use crate::utils::remote::{
    build_icechunk_s3_options, parse_remote_storage_url, register_standard_virtual_chunk_containers,
};
use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncStorageAdapter;
use zarrs_icechunk::AsyncIcechunkStore;

static ICECHUNK_STORE_CACHE: OnceLock<RwLock<HashMap<String, ReadableWritableListableStorage>>> =
    OnceLock::new();

/// Helper function to build a synchronous Zarr storage adapter over an Icechunk repository.
/// By default, opens a readonly session for the "main" branch. Caches stores by URL location.
pub fn build_sync_icechunk_store(
    location: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error + Send + Sync>> {
    let cache_lock = ICECHUNK_STORE_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let cache = cache_lock.read().unwrap_or_else(|p| p.into_inner());
    if let Some(store) = cache.get(location) {
        return Ok(store.clone());
    }
    drop(cache);

    let rt = get_shared_tokio_rt();

    let async_store = rt.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            let expanded = crate::utils::expand_tilde(location);
            let (storage, repo_config, auth_map) = if expanded.exists() {
                let storage = icechunk::new_local_filesystem_storage(&expanded).await?;
                (storage, None, HashMap::new())
            } else {
                let parsed = parse_remote_storage_url(location)?;
                let config = build_icechunk_s3_options(&parsed, true);

                let mut repo_config = icechunk::config::RepositoryConfig::default();
                let mut auth_map: HashMap<String, Option<icechunk::config::Credentials>> =
                    HashMap::new();

                register_standard_virtual_chunk_containers(
                    &mut repo_config,
                    &mut auth_map,
                    &config,
                    &parsed.bucket,
                );

                let storage = icechunk::new_s3_object_store_storage(
                    config,
                    parsed.bucket,
                    parsed.prefix,
                    None,
                    Vec::new(),
                    Vec::new(),
                )
                .await
                .map_err(|e| format!("Failed to create S3 storage for Icechunk: {e}"))?;

                (storage, Some(repo_config), auth_map)
            };

            let repo = icechunk::Repository::open(repo_config, storage, auth_map)
                .await
                .map_err(|e| format!("Failed to open Icechunk repository: {e}"))?;

            let version_info = icechunk::repository::VersionInfo::BranchTipRef("main".to_string());
            let session = repo
                .readonly_session(&version_info)
                .await
                .map_err(|e| format!("Failed to open readonly session on branch 'main': {e}"))?;

            let ice_store = Arc::new(AsyncIcechunkStore::new(session));
            Ok::<_, Box<dyn Error + Send + Sync>>(ice_store)
        })
        .await
        .map_err(|_| "Icechunk repository connection timed out after 30 seconds")?
    })?;

    let sync_store: ReadableWritableListableStorage = Arc::new(AsyncToSyncStorageAdapter::new(
        async_store,
        TokioBlockOn(rt),
    ));

    let mut cache = cache_lock.write().unwrap_or_else(|p| p.into_inner());
    cache.insert(location.to_string(), sync_store.clone());
    drop(cache);

    Ok(sync_store)
}

pub struct IcechunkBlockStore {
    inner: GenericZarrBlockStore,
}

impl IcechunkBlockStore {
    pub fn new(storage: ReadableWritableListableStorage, source_url: impl Into<String>) -> Self {
        Self {
            inner: GenericZarrBlockStore::new(storage, source_url, "icechunk", "Icechunk"),
        }
    }

    pub fn source_url(&self) -> &str {
        self.inner.source_url()
    }

    pub fn open(location: &str) -> Result<Self, BlockStoreError> {
        let storage = build_sync_icechunk_store(location).map_err(|e| e.to_string())?;
        Ok(Self::new(storage, location.trim_end_matches('/')))
    }
}

impl BlockStore for IcechunkBlockStore {
    fn backend_name(&self) -> &str {
        self.inner.backend_name()
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.inner.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block(request)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block_with_progress(request, on_progress)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        self.inner.fetch_blocks(requests)
    }
}
