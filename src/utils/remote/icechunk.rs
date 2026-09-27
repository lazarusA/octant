//! Icechunk S3 options builder and virtual chunk container registration.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

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
