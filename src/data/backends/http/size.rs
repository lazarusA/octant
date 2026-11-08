//! The size of a remote object, from a `HEAD` request's `Content-Length`:
//! readable by the page on any CORS-enabled server and, with no `Range`
//! header, sent without a preflight (suffix ranges need one, which some
//! stores refuse). In the browser, a server whose CORS rules allow only `GET`
//! (S3 buckets such as NOAA's) refuses the `HEAD`: the size then comes from a
//! plain `GET`, its body cancelled once the headers arrive.

/// The size in bytes of the object at `url`; an `HTTP 404 ...` error when
/// it is absent ([`super::is_not_found`]).
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_size(url: &str) -> Result<u64, String> {
    if url.starts_with(crate::data::backends::local::SCHEME) {
        return crate::data::backends::local::size(url).await;
    }
    let init = web_sys::RequestInit::new();
    init.set_method("HEAD");
    let resp = match super::fetch::send(url, &init).await {
        Ok(resp) => resp,
        Err(head_error) => {
            let resp = super::fetch::send(url, &web_sys::RequestInit::new())
                .await
                .map_err(|_| head_error)?;
            if let Some(body) = resp.body() {
                let _ = body.cancel();
            }
            resp
        }
    };
    if !resp.ok() {
        return Err(format!(
            "HTTP {} fetching the size of '{url}'",
            resp.status()
        ));
    }
    let length = resp.headers().get("Content-Length").ok().flatten();
    length
        .and_then(|l| l.trim().parse().ok())
        .ok_or_else(|| format!("No Content-Length for '{url}'"))
}

/// The size in bytes of the object at `url`, on desktop targets via `reqwest`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_url_size(url: &str) -> Result<u64, String> {
    let resp = super::fetch::get_http_client()
        .head(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "HTTP {} fetching the size of '{url}'",
            resp.status()
        ));
    }
    let length = resp.headers().get(reqwest::header::CONTENT_LENGTH);
    match length.and_then(|l| l.to_str().ok()?.trim().parse().ok()) {
        Some(length) => Ok(length),
        None => ranged_size(url).await,
    }
}

/// The size of the object at `url` from a one-byte ranged `GET`'s `Content-Range`
/// (`bytes 0-0/N`), for servers whose `HEAD` gives no length (THREDDS).
#[cfg(not(target_arch = "wasm32"))]
async fn ranged_size(url: &str) -> Result<u64, String> {
    let resp = (super::fetch::get_http_client().get(url))
        .header(reqwest::header::RANGE, "bytes=0-0")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let range = resp.headers().get(reqwest::header::CONTENT_RANGE);
    range
        .and_then(|r| r.to_str().ok()?.rsplit_once('/')?.1.trim().parse().ok())
        .ok_or_else(|| format!("No Content-Length for '{url}'"))
}
