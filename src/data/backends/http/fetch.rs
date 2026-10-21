//! HTTP byte fetching and range requests.

/// Whether `err`, from the fetches below, is an HTTP 404: the key is absent.
pub fn is_not_found(err: &str) -> bool {
    err.starts_with("HTTP 404 ")
}

/// Whether a metadata key looks absent: [`is_not_found`], or the 403 S3
/// answers for a missing key without list access. Not for chunks, where a 403
/// may be a store the user cannot read and must not show as empty data.
pub fn is_missing(err: &str) -> bool {
    is_not_found(err) || err.starts_with("HTTP 403 ")
}

/// How long a browser request waits for the server's answer (its headers).
#[cfg(target_arch = "wasm32")]
const ANSWER_TIMEOUT_MS: i32 = 60_000;

/// Sends `init` to `url` and waits for the answer, giving up after
/// [`ANSWER_TIMEOUT_MS`]: `fetch` has no limit of its own, and a server that
/// never answers would hold its prefetch worker for good. Only the wait for
/// the answer is limited, so a slow but steady body still arrives.
/// (`AbortController` and `setTimeout`, unlike `AbortSignal.timeout`, exist
/// in every browser the WebGL build runs in.)
#[cfg(target_arch = "wasm32")]
pub(crate) async fn send(
    url: &str,
    init: &web_sys::RequestInit,
) -> Result<web_sys::Response, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_futures::JsFuture;

    /// The pending timer, cleared when the answer comes or the fetch is
    /// dropped, so it never calls its dropped closure.
    struct Timer {
        window: web_sys::Window,
        handle: i32,
        _abort: Closure<dyn FnMut()>,
    }
    impl Drop for Timer {
        fn drop(&mut self) {
            self.window.clear_timeout_with_handle(self.handle);
        }
    }

    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;
    let controller =
        web_sys::AbortController::new().map_err(|e| format!("No AbortController: {e:?}"))?;
    init.set_signal(Some(&controller.signal()));
    let request = web_sys::Request::new_with_str_and_init(url, init)
        .map_err(|e| format!("Failed to build request: {e:?}"))?;
    let abort = Closure::<dyn FnMut()>::new(move || controller.abort());
    let handle = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            abort.as_ref().unchecked_ref(),
            ANSWER_TIMEOUT_MS,
        )
        .map_err(|e| format!("Failed to set a timeout: {e:?}"))?;
    let _timer = Timer {
        window: window.clone(),
        handle,
        _abort: abort,
    };
    JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Network fetch failed for '{url}': {e:?}"))?
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())
}

/// Fetches raw bytes from a remote URL using browser `window.fetch()`.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;

    let resp = send(url, &web_sys::RequestInit::new()).await?;

    if !resp.ok() {
        return Err(format!("HTTP {} fetching '{}'", resp.status(), url));
    }

    let array_buffer_prom = resp
        .array_buffer()
        .map_err(|e| format!("Failed to read array buffer: {e:?}"))?;

    let array_buffer_val = JsFuture::from(array_buffer_prom)
        .await
        .map_err(|e| format!("Failed to resolve array buffer: {e:?}"))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer_val);
    Ok(uint8_array.to_vec())
}

/// Fetches a specific byte range from a remote URL using HTTP Range headers.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_byte_range(url: &str, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;

    let headers =
        web_sys::Headers::new().map_err(|e| format!("Failed to create headers: {e:?}"))?;
    let end = offset.saturating_add(length).saturating_sub(1);
    let range_val = format!("bytes={offset}-{end}");
    headers
        .set("Range", &range_val)
        .map_err(|e| format!("Failed to set Range header: {e:?}"))?;

    let init = web_sys::RequestInit::new();
    init.set_headers(&headers);
    let resp = send(url, &init).await?;

    let status = resp.status();
    if status != 200 && status != 206 {
        return Err(format!("HTTP {status} fetching byte range from '{url}'"));
    }

    let array_buffer_prom = resp
        .array_buffer()
        .map_err(|e| format!("Failed to read array buffer: {e:?}"))?;

    let array_buffer_val = JsFuture::from(array_buffer_prom)
        .await
        .map_err(|e| format!("Failed to resolve array buffer: {e:?}"))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer_val);
    let bytes = uint8_array.to_vec();
    if status == 200 && (bytes.len() as u64) > length {
        let start = offset as usize;
        let slice_end = (offset + length) as usize;
        if start < bytes.len() {
            return Ok(bytes[start..slice_end.min(bytes.len())].to_vec());
        }
    }
    Ok(bytes)
}

#[cfg(not(target_arch = "wasm32"))]
static HTTP_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default()
    })
}

/// Fetches raw bytes from a remote URL on desktop targets via `reqwest`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = get_http_client();
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    // A missing key (404) is an error, as in the browser, not its error page.
    if !resp.status().is_success() {
        return Err(format!("HTTP {} fetching '{}'", resp.status(), url));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

/// Fetches a specific byte range from a remote URL on desktop targets via `reqwest`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_url_byte_range(url: &str, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    let client = get_http_client();
    let end = offset.saturating_add(length).saturating_sub(1);
    let resp = client
        .get(url)
        .header("Range", format!("bytes={offset}-{end}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status().as_u16();
    if status != 200 && status != 206 {
        return Err(format!("HTTP {status} fetching byte range from '{url}'"));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
    if status == 200 && (bytes.len() as u64) > length {
        let start = offset as usize;
        let slice_end = (offset + length) as usize;
        if start < bytes.len() {
            return Ok(bytes[start..slice_end.min(bytes.len())].to_vec());
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::is_missing;

    #[test]
    fn missing_keys_are_404_and_403() {
        assert!(is_missing("HTTP 404 fetching 'a/.zattrs'"));
        assert!(is_missing("HTTP 403 Forbidden fetching 'a/0.0'"));
        assert!(!is_missing("HTTP 500 fetching 'a/0.0'"));
        assert!(!is_missing("Network fetch failed for 'a/0.0'"));
        assert!(!super::is_not_found("HTTP 403 Forbidden fetching 'a/0.0'"));
    }
}
