//! Icechunk S3 options builder and virtual chunk container registration.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use super::url::ParsedStorageUrl;

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

#[cfg(not(target_arch = "wasm32"))]
/// Registers standard open-data and virtual chunk container prefixes in the Icechunk repository configuration.
pub fn register_standard_virtual_chunk_containers(
    repo_config: &mut icechunk::config::RepositoryConfig,
    auth_map: &mut HashMap<String, Option<icechunk::config::Credentials>>,
    base_s3_opts: &icechunk::config::S3Options,
    primary_bucket: &str,
) {
    let mut virt_opts = base_s3_opts.clone();
    virt_opts.endpoint_url = None;
    virt_opts.force_path_style = false;

    for pfx in known_prefixes(primary_bucket) {
        let mut opts = virt_opts.clone();
        let bucket = pfx
            .trim_start_matches("s3://")
            .trim_start_matches("https://");
        if let Some(region) = region_of(bucket.split(['/', '.']).next().unwrap_or(bucket)) {
            opts.region = Some(region.to_owned());
        }
        if let Ok(container) = icechunk::virtual_chunks::VirtualChunkContainer::new(
            pfx.clone(),
            icechunk::config::ObjectStoreConfig::S3(opts),
        ) {
            let _ = repo_config.set_virtual_chunk_container(container);
        }
        auth_map.insert(pfx, None);
    }
}

/// The prefixes of the repository's own bucket and of the open-data buckets virtual chunks
/// commonly live in, with catch-alls.
#[cfg(not(target_arch = "wasm32"))]
fn known_prefixes(primary_bucket: &str) -> Vec<String> {
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
        "noaa-gfs-bdp-pds/".to_string(),
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
    known_prefixes
}

/// The region of a public bucket virtual chunks often live in, which may not be the
/// repository's: S3 answers a request sent to another region with a redirect, which
/// object_store does not follow (NOAA's HRRR GRIB files, under dynamical.org's repo).
#[cfg(not(target_arch = "wasm32"))]
fn region_of(bucket: &str) -> Option<&'static str> {
    match bucket {
        "dynamical-noaa-hrrr" => Some("us-west-2"),
        b if b.starts_with("copernicus-dem-") => Some("eu-central-1"),
        b if b.starts_with("noaa-") => Some("us-east-1"),
        _ => None,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    /// Virtual chunks in another region than their repository's are read in their own
    /// (dynamical.org's HRRR repo in us-west-2, NOAA's GRIB files in us-east-1).
    #[test]
    fn known_buckets_have_their_region() {
        use super::region_of;
        assert_eq!(region_of("noaa-hrrr-bdp-pds"), Some("us-east-1"));
        assert_eq!(region_of("dynamical-noaa-hrrr"), Some("us-west-2"));
        assert_eq!(region_of("copernicus-dem-30m"), Some("eu-central-1"));
        assert_eq!(region_of("someone-elses-bucket"), None);
    }
}
