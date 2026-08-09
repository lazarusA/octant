//! Storage initializers for Icechunk repositories.

use crate::utils::executor::{TokioBlockOn, get_shared_tokio_rt};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};
use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncStorageAdapter;
use zarrs_icechunk::AsyncIcechunkStore;

static ICECHUNK_STORE_CACHE: OnceLock<RwLock<HashMap<String, ReadableWritableListableStorage>>> =
    OnceLock::new();

/// Helper function to build a synchronous Zarr storage adapter over an Icechunk repository.
/// By default, opens a readonly session for the "main" branch. Caches stores by URL location.
pub fn build_sync_icechunk_store(
    location: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error>> {
    let cache_lock = ICECHUNK_STORE_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    if let Ok(cache) = cache_lock.read()
        && let Some(store) = cache.get(location)
    {
        return Ok(store.clone());
    }

    let rt = get_shared_tokio_rt();

    let async_store = rt.block_on(async {
        let path = Path::new(location);
        let storage = if path.exists() {
            icechunk::new_local_filesystem_storage(path).await?
        } else {
            let (bucket, prefix, region, endpoint_url) = parse_s3_or_http_url(location)?;
            let mut config = icechunk::config::S3Options::default();
            config.region = region.or_else(|| Some("us-east-1".to_string()));
            config.endpoint_url = endpoint_url;
            config.anonymous = true;
            config.allow_http = true;
            config.force_path_style = true;

            icechunk::new_s3_object_store_storage(
                config,
                bucket,
                prefix,
                None,
                Vec::new(),
                Vec::new(),
            )
            .await?
        };

        let repo = icechunk::Repository::open(None, storage, Default::default()).await?;
        let version_info = icechunk::repository::VersionInfo::BranchTipRef("main".to_string());
        let session = repo.readonly_session(&version_info).await?;
        let ice_store = Arc::new(AsyncIcechunkStore::new(session));
        Ok::<_, Box<dyn Error>>(ice_store)
    })?;

    let sync_store: ReadableWritableListableStorage = Arc::new(AsyncToSyncStorageAdapter::new(
        async_store,
        TokioBlockOn(rt.clone()),
    ));

    if let Ok(mut cache) = cache_lock.write() {
        cache.insert(location.to_string(), sync_store.clone());
    }

    Ok(sync_store)
}

#[allow(clippy::type_complexity)]
pub fn parse_s3_or_http_url(
    url: &str,
) -> Result<(String, Option<String>, Option<String>, Option<String>), Box<dyn Error>> {
    let clean = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err("Invalid storage URL".into());
    }

    let host = parts[0];
    let path_parts = &parts[1..];

    if host.contains(".s3.") && host.ends_with(".amazonaws.com") {
        let sub_parts: Vec<&str> = host
            .trim_end_matches(".amazonaws.com")
            .split(".s3.")
            .collect();
        let bucket = sub_parts.first().unwrap_or(&host).to_string();
        let region = sub_parts.get(1).map(|r| r.to_string());
        let prefix = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join("/"))
        };
        Ok((bucket, prefix, region, None))
    } else if host.contains(".s3-") && host.ends_with(".amazonaws.com") {
        let sub_parts: Vec<&str> = host
            .trim_end_matches(".amazonaws.com")
            .split(".s3-")
            .collect();
        let bucket = sub_parts.first().unwrap_or(&host).to_string();
        let region = sub_parts.get(1).map(|r| r.to_string());
        let prefix = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join("/"))
        };
        Ok((bucket, prefix, region, None))
    } else if host == "data.source.coop" {
        if path_parts.is_empty() {
            return Err("Missing bucket in source.coop URL".into());
        }
        let bucket = path_parts[0].to_string();
        let prefix = if path_parts.len() > 1 {
            Some(path_parts[1..].join("/"))
        } else {
            None
        };
        let endpoint_url = Some("https://data.source.coop".to_string());
        Ok((bucket, prefix, None, endpoint_url))
    } else {
        let bucket = path_parts.first().copied().unwrap_or(host).to_string();
        let prefix = if path_parts.len() > 1 {
            Some(path_parts[1..].join("/"))
        } else {
            None
        };
        let scheme = if url.starts_with("http://") {
            "http"
        } else {
            "https"
        };
        let endpoint_url = Some(format!("{}://{}", scheme, host));
        Ok((bucket, prefix, None, endpoint_url))
    }
}
