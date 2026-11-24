//! Browser reads of public GCS buckets without a CORS policy (ARCO-ERA5).
//! `storage.googleapis.com/bucket/key` sends CORS headers only for buckets
//! with a policy; GCS's JSON media endpoint sends them for any origin. A
//! bucket moves to that endpoint once its plain URL is blocked
//! (`fetch::send`), so buckets with CORS keep their plain URLs.

#[cfg(target_arch = "wasm32")]
use std::{borrow::Cow, cell::RefCell, collections::HashSet};

const HOST: &str = "https://storage.googleapis.com/";

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// Buckets whose plain URLs the browser blocked, per page or worker.
    static NO_CORS: RefCell<HashSet<String>> = RefCell::default();
}

/// The bucket and key of a path-style GCS object URL without a query.
fn split(url: &str) -> Option<(&str, &str)> {
    let path = crate::utils::remote::strip_scheme(url, HOST)?;
    let (bucket, key) = path.split_once('/')?;
    let plain = !key.is_empty() && !url.contains('?');
    (plain && bucket != "download" && bucket != "storage").then_some((bucket, key))
}

/// `url` at the JSON media endpoint; `None` for URLs that are not GCS objects.
pub(super) fn cors_url(url: &str) -> Option<String> {
    use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
    const KEY: &percent_encoding::AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~');
    let (bucket, key) = split(url)?;
    let key = percent_decode_str(key).decode_utf8().ok()?;
    let key = utf8_percent_encode(&key, KEY);
    Some(format!(
        "{HOST}download/storage/v1/b/{bucket}/o/{key}?alt=media"
    ))
}

/// `url` as first sent: at the JSON endpoint when its bucket was blocked before.
#[cfg(target_arch = "wasm32")]
pub(super) fn known(url: &str) -> Cow<'_, str> {
    let blocked = split(url).is_some_and(|(b, _)| NO_CORS.with_borrow(|s| s.contains(b)));
    match blocked.then(|| cors_url(url)).flatten() {
        Some(json) => json.into(),
        None => url.into(),
    }
}

/// Sends `url`'s bucket to the JSON endpoint from now on.
#[cfg(target_arch = "wasm32")]
pub(super) fn remember(url: &str) {
    if let Some((bucket, _)) = split(url) {
        NO_CORS.with_borrow_mut(|s| s.insert(bucket.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::cors_url;

    #[test]
    fn gcs_objects_map_to_the_json_media_endpoint() {
        assert_eq!(
            cors_url(
                "https://storage.googleapis.com/gcp-public-data-arco-era5/ar/x.zarr-v3/.zmetadata"
            )
            .as_deref(),
            Some(
                "https://storage.googleapis.com/download/storage/v1/b/gcp-public-data-arco-era5/o/ar%2Fx.zarr-v3%2F.zmetadata?alt=media"
            )
        );
        // An already-encoded key is not encoded twice.
        assert_eq!(
            cors_url("https://storage.googleapis.com/b/my%20data.zarr/.zattrs").as_deref(),
            Some(
                "https://storage.googleapis.com/download/storage/v1/b/b/o/my%20data.zarr%2F.zattrs?alt=media"
            )
        );
        for kept in [
            "https://storage.googleapis.com/bucket",
            "https://storage.googleapis.com/bucket/",
            "https://storage.googleapis.com/download/storage/v1/b/b/o/k?alt=media",
            "https://storage.googleapis.com/b/k?x=1",
            "https://b.s3.amazonaws.com/k",
        ] {
            assert_eq!(cors_url(kept), None, "{kept}");
        }
    }
}
