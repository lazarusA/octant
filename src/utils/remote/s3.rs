//! S3 buckets read anonymously over HTTPS: a bucket's region, and its
//! regional path-style URL.

/// The AWS region of `bucket`, from the `x-amz-bucket-region` header S3 sends
/// any anonymous request, or `None` when the bucket does not answer. Asked
/// path-style, so bucket names with dots (`us-west-2.opendata.source.coop`)
/// still match S3's certificate.
pub async fn discover_bucket_region(bucket: &str) -> Option<String> {
    let url = format!("https://s3.amazonaws.com/{bucket}/");
    let resp = crate::data::backends::http::get_http_client()
        .head(&url)
        .send()
        .await
        .ok()?;
    let region = resp.headers().get("x-amz-bucket-region")?.to_str().ok()?;
    Some(region.to_string())
}

/// `s3://bucket/key` as `https://s3.{region}.amazonaws.com/bucket/key`: S3
/// answers a wrong region with a redirect, and a virtual-hosted name with
/// dots fails TLS. Other URLs are returned unchanged.
pub fn s3_as_https(url: &str) -> String {
    let Some(rest) = url.strip_prefix("s3://") else {
        return url.to_string();
    };
    let (bucket, key) = rest.split_once('/').unwrap_or((rest, ""));
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let region = rt
        .block_on(discover_bucket_region(bucket))
        .unwrap_or_else(|| "us-east-1".to_string());
    format!("https://s3.{region}.amazonaws.com/{bucket}/{key}")
        .trim_end_matches('/')
        .to_string()
}
