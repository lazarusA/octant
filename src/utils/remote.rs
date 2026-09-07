//! Remote storage URL parsing, S3 configuration, and virtual chunk container helpers.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::error::Error;

/// Structured components of a remote S3 or HTTP object storage location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStorageUrl {
    /// Target bucket or root container identifier.
    pub bucket: String,
    /// Sub-path or prefix inside the bucket.
    pub prefix: Option<String>,
    /// AWS region (e.g. `us-east-1`, `us-west-2`), if specified or inferred from the host.
    pub region: Option<String>,
    /// Custom HTTP(S) endpoint URL (e.g. for MinIO, Source Cooperative, Wasabi), if applicable.
    pub endpoint_url: Option<String>,
    /// Whether path-style addressing is forced for the S3 client.
    pub force_path_style: bool,
}

/// Parses any S3 URL, virtual-hosted S3 endpoint, Source Cooperative endpoint,
/// or custom S3/HTTP object store address into structured components.
pub fn parse_remote_storage_url(
    url: &str,
) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Storage URL is empty".into());
    }

    if let Some(s3_path) = trimmed.strip_prefix("s3://") {
        let parts: Vec<&str> = s3_path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Err("Missing bucket name in s3:// URL".into());
        }
        let bucket = parts[0].to_string();
        let prefix = if parts.len() > 1 {
            Some(parts[1..].join("/"))
        } else {
            None
        };
        return Ok(ParsedStorageUrl {
            bucket,
            prefix,
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
            force_path_style: false,
        });
    }

    let clean = trimmed
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
        Ok(ParsedStorageUrl {
            bucket,
            prefix,
            region,
            endpoint_url: None,
            force_path_style: false,
        })
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
        Ok(ParsedStorageUrl {
            bucket,
            prefix,
            region,
            endpoint_url: None,
            force_path_style: false,
        })
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
        Ok(ParsedStorageUrl {
            bucket,
            prefix,
            region: Some("us-east-1".to_string()),
            endpoint_url,
            force_path_style: true,
        })
    } else {
        let bucket = path_parts.first().copied().unwrap_or(host).to_string();
        let prefix = if path_parts.len() > 1 {
            Some(path_parts[1..].join("/"))
        } else {
            None
        };
        let scheme = if trimmed.starts_with("http://") {
            "http"
        } else {
            "https"
        };
        let endpoint_url = Some(format!("{}://{}", scheme, host));
        Ok(ParsedStorageUrl {
            bucket,
            prefix,
            region: Some("us-east-1".to_string()),
            endpoint_url,
            force_path_style: true,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Builds standard `icechunk::config::S3Options` configured with timeouts and path styling.
pub fn build_icechunk_s3_options(
    parsed: &ParsedStorageUrl,
    anonymous: bool,
) -> icechunk::config::S3Options {
    let mut config = icechunk::config::S3Options::default();
    config.region = parsed
        .region
        .clone()
        .or_else(|| Some("us-east-1".to_string()));
    config.endpoint_url = parsed.endpoint_url.clone();
    config.anonymous = anonymous;
    config.allow_http = true;
    config.force_path_style = parsed.force_path_style;
    config
}

/// Registers standard open-data and virtual chunk container prefixes in the Icechunk repository configuration.
#[cfg(not(target_arch = "wasm32"))]
pub fn register_standard_virtual_chunk_containers(
    repo_config: &mut icechunk::config::RepositoryConfig,
    auth_map: &mut HashMap<String, Option<icechunk::config::Credentials>>,
    base_s3_opts: &icechunk::config::S3Options,
    primary_bucket: &str,
) {
    let mut virt_opts = base_s3_opts.clone();
    virt_opts.endpoint_url = None;
    virt_opts.force_path_style = false;

    let mut known_prefixes = vec![
        primary_bucket.to_string(),
        format!("{}/", primary_bucket),
        format!("s3://{}", primary_bucket),
        format!("s3://{}/", primary_bucket),
        "noaa-cdr-ndvi-pds".to_string(),
        "noaa-cdr-ndvi-pds/".to_string(),
        "s3://noaa-cdr-ndvi-pds".to_string(),
        "s3://noaa-cdr-ndvi-pds/".to_string(),
        "https://noaa-cdr-ndvi-pds.s3.amazonaws.com/".to_string(),
        "https://noaa-cdr-ndvi-pds.s3.us-east-1.amazonaws.com/".to_string(),
        "dynamical-noaa-hrrr".to_string(),
        "dynamical-noaa-hrrr/".to_string(),
        "s3://dynamical-noaa-hrrr".to_string(),
        "s3://dynamical-noaa-hrrr/".to_string(),
        "noaa-hrrr-bdp-pds".to_string(),
        "noaa-hrrr-bdp-pds/".to_string(),
        "s3://noaa-hrrr-bdp-pds".to_string(),
        "s3://noaa-hrrr-bdp-pds/".to_string(),
        "noaa-goes16".to_string(),
        "s3://noaa-goes16/".to_string(),
        "noaa-goes17".to_string(),
        "s3://noaa-goes17/".to_string(),
        "noaa-goes18".to_string(),
        "s3://noaa-goes18/".to_string(),
        "noaa-gfs-bdp-pds".to_string(),
        "s3://noaa-gfs-bdp-pds/".to_string(),
        "noaa-nwm-pds".to_string(),
        "s3://noaa-nwm-pds/".to_string(),
        "copernicus-dem-30m".to_string(),
        "s3://copernicus-dem-30m/".to_string(),
        "copernicus-dem-90m".to_string(),
        "s3://copernicus-dem-90m/".to_string(),
        "s3://".to_string(),
        "s3".to_string(),
        "virtual".to_string(),
        "default".to_string(),
    ];

    known_prefixes.dedup();

    for pfx in known_prefixes {
        if let Ok(container) = icechunk::virtual_chunks::VirtualChunkContainer::new(
            pfx.clone(),
            icechunk::config::ObjectStoreConfig::S3(virt_opts.clone()),
        ) {
            let _ = repo_config.set_virtual_chunk_container(container);
        }
        auth_map.insert(pfx, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aws_virtual_hosted_url() {
        let url = "https://dynamical-noaa-hrrr.s3.us-west-2.amazonaws.com/noaa-hrrr-forecast-48-hour-virtual/v0.5.0.icechunk/";
        let parsed = parse_remote_storage_url(url).unwrap();
        assert_eq!(parsed.bucket, "dynamical-noaa-hrrr");
        assert_eq!(
            parsed.prefix,
            Some("noaa-hrrr-forecast-48-hour-virtual/v0.5.0.icechunk".to_string())
        );
        assert_eq!(parsed.region, Some("us-west-2".to_string()));
        assert_eq!(parsed.endpoint_url, None);
        assert!(!parsed.force_path_style);
    }

    #[test]
    fn test_parse_source_coop_url() {
        let url = "https://data.source.coop/eeholmes/chlaz/icechunk";
        let parsed = parse_remote_storage_url(url).unwrap();
        assert_eq!(parsed.bucket, "eeholmes");
        assert_eq!(parsed.prefix, Some("chlaz/icechunk".to_string()));
        assert_eq!(
            parsed.endpoint_url,
            Some("https://data.source.coop".to_string())
        );
        assert!(parsed.force_path_style);
    }

    #[test]
    fn test_parse_s3_uri() {
        let url = "s3://my-bucket/dataset.zarr";
        let parsed = parse_remote_storage_url(url).unwrap();
        assert_eq!(parsed.bucket, "my-bucket");
        assert_eq!(parsed.prefix, Some("dataset.zarr".to_string()));
    }
}
