use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use zarrs::storage::storage_adapter::async_to_sync::{AsyncToSyncBlockOn, AsyncToSyncStorageAdapter};
use zarrs::storage::ReadableWritableListableStorage;
use zarrs_icechunk::AsyncIcechunkStore;

struct TokioBlockOn(Arc<tokio::runtime::Runtime>);

impl AsyncToSyncBlockOn for TokioBlockOn {
    fn block_on<F: core::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

/// Helper function to build a synchronous Zarr storage adapter over an Icechunk repository.
/// By default, opens a readonly session for the "main" branch.
pub fn build_sync_icechunk_store(
    location: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error>> {
    println!("[ICECHUNK DEBUG] Opening store at location: {}", location);
    let rt = Arc::new(tokio::runtime::Builder::new_current_thread().enable_all().build()?);

    let async_store = rt.block_on(async {
        let path = Path::new(location);
        let storage = if path.exists() {
            println!("[ICECHUNK DEBUG] Opening local filesystem storage: {:?}", path);
            icechunk::new_local_filesystem_storage(path).await?
        } else {
            let (bucket, prefix, region, endpoint_url) = parse_s3_or_http_url(location)?;
            println!(
                "[ICECHUNK DEBUG] Parsed URL -> bucket: '{}', prefix: '{:?}', region: '{:?}', endpoint_url: '{:?}'",
                bucket, prefix, region, endpoint_url
            );

            let mut config = icechunk::config::S3Options::default();
            config.region = region.or_else(|| Some("us-east-1".to_string()));
            config.endpoint_url = endpoint_url;
            config.anonymous = true;
            config.allow_http = true;
            config.force_path_style = true;

            println!("[ICECHUNK DEBUG] Creating S3 storage adapter...");
            icechunk::new_s3_object_store_storage(
                config,
                bucket,
                prefix,
                None,
                Vec::new(),
                Vec::new(),
            ).await?
        };

        println!("[ICECHUNK DEBUG] Opening Icechunk Repository...");
        let repo = match icechunk::Repository::open(None, storage, Default::default()).await {
            Ok(r) => {
                println!("[ICECHUNK DEBUG] Repository opened successfully!");
                r
            }
            Err(e) => {
                eprintln!("[ICECHUNK DEBUG ERROR] Failed to open Repository: {:?}", e);
                return Err(Box::new(e) as Box<dyn Error>);
            }
        };

        println!("[ICECHUNK DEBUG] Opening readonly session for branch 'main'...");
        let version_info = icechunk::repository::VersionInfo::BranchTipRef("main".to_string());
        let session = match repo.readonly_session(&version_info).await {
            Ok(s) => {
                println!("[ICECHUNK DEBUG] Readonly session opened successfully!");
                s
            }
            Err(e) => {
                eprintln!("[ICECHUNK DEBUG ERROR] Failed to open readonly_session for 'main': {:?}", e);
                return Err(Box::new(e) as Box<dyn Error>);
            }
        };

        let ice_store = Arc::new(AsyncIcechunkStore::new(session));
        println!("[ICECHUNK DEBUG] AsyncIcechunkStore created.");
        Ok::<_, Box<dyn Error>>(ice_store)
    })?;

    let sync_store: ReadableWritableListableStorage = Arc::new(AsyncToSyncStorageAdapter::new(
        async_store,
        TokioBlockOn(rt.clone()),
    ));

    println!("[ICECHUNK DEBUG] Synchronous storage adapter ready.");
    Ok(sync_store)
}

fn parse_s3_or_http_url(url: &str) -> Result<(String, Option<String>, Option<String>, Option<String>), Box<dyn Error>> {
    let clean = url.trim_start_matches("https://").trim_start_matches("http://");
    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err("Invalid storage URL".into());
    }

    let host = parts[0];
    let path_parts = &parts[1..];

    if host.contains(".s3.") && host.ends_with(".amazonaws.com") {
        // E.g. dynamical-noaa-hrrr.s3.us-west-2.amazonaws.com
        let sub_parts: Vec<&str> = host.trim_end_matches(".amazonaws.com").split(".s3.").collect();
        let bucket = sub_parts.first().unwrap_or(&host).to_string();
        let region = sub_parts.get(1).map(|r| r.to_string());
        let prefix = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join("/"))
        };
        Ok((bucket, prefix, region, None))
    } else if host.contains(".s3-") && host.ends_with(".amazonaws.com") {
        let sub_parts: Vec<&str> = host.trim_end_matches(".amazonaws.com").split(".s3-").collect();
        let bucket = sub_parts.first().unwrap_or(&host).to_string();
        let region = sub_parts.get(1).map(|r| r.to_string());
        let prefix = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join("/"))
        };
        Ok((bucket, prefix, region, None))
    } else if host == "data.source.coop" {
        // E.g. https://data.source.coop/e4drr-project/observations/chirps_daily_icechunk
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
        let scheme = if url.starts_with("http://") { "http" } else { "https" };
        let endpoint_url = Some(format!("{}://{}", scheme, host));
        Ok((bucket, prefix, None, endpoint_url))
    }
}

