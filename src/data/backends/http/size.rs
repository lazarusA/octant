//! The size of a remote object, from a `HEAD` request's `Content-Length`:
//! readable by the page on any CORS-enabled server and, with no `Range`
//! header, sent without a preflight (suffix ranges need one, which some
//! stores refuse).

/// The size in bytes of the object at `url`; an `HTTP 404 ...` error when
/// it is absent ([`super::is_not_found`]).
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_size(url: &str) -> Result<u64, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;
    let init = web_sys::RequestInit::new();
    init.set_method("HEAD");
    let request = web_sys::Request::new_with_str_and_init(url, &init)
        .map_err(|e| format!("Failed to build request: {e:?}"))?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Network fetch failed for '{url}': {e:?}"))?
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())?;
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
