//! The size of a remote object, from a `HEAD` request's `Content-Length`:
//! readable by the page on any CORS-enabled server and, with no `Range`
//! header, sent without a preflight (suffix ranges need one, which some
//! stores refuse).

/// The size in bytes of the object at `url`; an `HTTP 404 ...` error when
/// it is absent ([`super::is_not_found`]).
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_size(url: &str) -> Result<u64, String> {
    let init = web_sys::RequestInit::new();
    init.set_method("HEAD");
    let resp = super::fetch::send(url, &init).await?;
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
    length
        .and_then(|l| l.to_str().ok()?.trim().parse().ok())
        .ok_or_else(|| format!("No Content-Length for '{url}'"))
}
