//! Remote storage URL parsing, S3 components, and virtual chunk container HTTPS conversions.

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

/// Converts an S3 or Virtual Chunk Container URI into an HTTPS URL for browser fetching.
pub fn s3_to_https(location: &str) -> String {
    let clean = location.trim_start_matches("icechunk+");
    if let Some(rest) = clean.strip_prefix("s3://") {
        if let Some((bucket, key)) = rest.split_once('/') {
            format!("https://{bucket}.s3.amazonaws.com/{key}")
        } else {
            format!("https://{rest}.s3.amazonaws.com")
        }
    } else if let Some(rest) = clean.strip_prefix("vcc://") {
        if let Some((container, key)) = rest.split_once('/') {
            format!("https://{container}.s3.amazonaws.com/{key}")
        } else {
            format!("https://{rest}.s3.amazonaws.com")
        }
    } else {
        clean.to_string()
    }
}

/// Parses any S3 URL, virtual-hosted S3 endpoint, Source Cooperative endpoint,
/// or custom S3/HTTP object store address into structured components.
pub fn parse_remote_storage_url(
    url: &str,
) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
    let trimmed = url.trim().trim_start_matches("icechunk+");
    if trimmed.is_empty() {
        return Err("Storage URL is empty".into());
    }

    if let Some(s3_path) = trimmed
        .strip_prefix("s3://")
        .or_else(|| trimmed.strip_prefix("vcc://"))
    {
        return parse_s3_or_vcc_scheme(s3_path);
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
        parse_aws_virtual_hosted(host, path_parts, ".s3.")
    } else if host.contains(".s3-") && host.ends_with(".amazonaws.com") {
        parse_aws_virtual_hosted(host, path_parts, ".s3-")
    } else if host == "data.source.coop" {
        parse_source_coop(path_parts)
    } else {
        parse_generic_http_endpoint(trimmed, host, path_parts)
    }
}

fn parse_s3_or_vcc_scheme(s3_path: &str) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = s3_path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err("Missing bucket name in s3:// or vcc:// URL".into());
    }
    let bucket = parts[0].to_string();
    let prefix = if parts.len() > 1 {
        Some(parts[1..].join("/"))
    } else {
        None
    };
    Ok(ParsedStorageUrl {
        bucket,
        prefix,
        region: Some("us-east-1".to_string()),
        endpoint_url: None,
        force_path_style: false,
    })
}

fn parse_aws_virtual_hosted(
    host: &str,
    path_parts: &[&str],
    delimiter: &str,
) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
    let sub_parts: Vec<&str> = host
        .trim_end_matches(".amazonaws.com")
        .split(delimiter)
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
}

fn parse_source_coop(
    path_parts: &[&str],
) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
    if path_parts.is_empty() {
        return Err("Missing bucket in source.coop URL".into());
    }
    let bucket = path_parts[0].to_string();
    let prefix = if path_parts.len() > 1 {
        Some(path_parts[1..].join("/"))
    } else {
        None
    };
    Ok(ParsedStorageUrl {
        bucket,
        prefix,
        region: Some("us-east-1".to_string()),
        endpoint_url: Some("https://data.source.coop".to_string()),
        force_path_style: true,
    })
}

fn parse_generic_http_endpoint(
    trimmed: &str,
    host: &str,
    path_parts: &[&str],
) -> Result<ParsedStorageUrl, Box<dyn Error + Send + Sync>> {
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
