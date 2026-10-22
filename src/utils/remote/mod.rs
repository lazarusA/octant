//! Remote storage URL parsing, S3 configuration, and virtual chunk container helpers.

pub mod icechunk;
#[cfg(not(target_arch = "wasm32"))]
pub mod s3;
pub mod url;

#[cfg(not(target_arch = "wasm32"))]
pub use icechunk::{build_icechunk_s3_options, register_standard_virtual_chunk_containers};
#[cfg(not(target_arch = "wasm32"))]
pub use s3::{discover_bucket_region, s3_as_https};
pub use url::{ParsedStorageUrl, parse_remote_storage_url, s3_to_https};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_to_https_conversions() {
        assert_eq!(
            s3_to_https("s3://my-bucket/path/to/chunk"),
            "https://my-bucket.s3.amazonaws.com/path/to/chunk"
        );
        assert_eq!(
            s3_to_https("s3://simple-bucket"),
            "https://simple-bucket.s3.amazonaws.com"
        );
        assert_eq!(
            s3_to_https("vcc://container-bucket/virtual/key.nc"),
            "https://container-bucket.s3.amazonaws.com/virtual/key.nc"
        );
        assert_eq!(
            s3_to_https("https://direct.domain.com/data.zarr"),
            "https://direct.domain.com/data.zarr"
        );
        assert_eq!(
            s3_to_https("icechunk+s3://icechunk-bucket/repo"),
            "https://icechunk-bucket.s3.amazonaws.com/repo"
        );
    }

    #[test]
    fn test_parse_aws_virtual_hosted_url() {
        let url = "https://dynamical-noaa-hrrr.s3.us-west-2.amazonaws.com/noaa-hrrr-forecast-48-hour-virtual/v0.5.0.icechunk/";
        let parsed = parse_remote_storage_url(url).expect("Should parse AWS virtual hosted url");
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
    fn test_parse_aws_virtual_hosted_url_without_region() {
        let url = "https://tessera-embeddings.s3.amazonaws.com/v1.1/dclimate.icechunk/";
        let parsed = parse_remote_storage_url(url).expect("Should parse AWS virtual hosted url");
        assert_eq!(parsed.bucket, "tessera-embeddings");
        assert_eq!(parsed.prefix, Some("v1.1/dclimate.icechunk".to_string()));
        assert_eq!(parsed.region, None);
    }

    #[test]
    fn test_parse_source_coop_url() {
        let url = "https://data.source.coop/eeholmes/chlaz/icechunk";
        let parsed = parse_remote_storage_url(url).expect("Should parse Source Coop url");
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
        let parsed = parse_remote_storage_url(url).expect("Should parse s3 uri");
        assert_eq!(parsed.bucket, "my-bucket");
        assert_eq!(parsed.prefix, Some("dataset.zarr".to_string()));
        assert_eq!(parsed.region, None);
    }

    /// `cargo test --lib icechunk_outside_us_east_1_opens -- --ignored`: the
    /// TESSERA store lives in us-west-2, and neither URL names the region.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "network"]
    fn icechunk_outside_us_east_1_opens() {
        use crate::data::blocks::BlockStore;
        for url in [
            "https://tessera-embeddings.s3.amazonaws.com/v1.1/dclimate.icechunk/",
            "s3://tessera-embeddings/v1.1/dclimate.icechunk/",
        ] {
            let store = crate::data::backends::IcechunkBlockStore::open(url).expect(url);
            let vars = store.variables().expect(url);
            assert!(vars.iter().any(|v| v.ends_with("embeddings")), "{url}");
        }
    }
}
