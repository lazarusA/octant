//! HTTP byte fetching and range requests.

/// Fetches raw bytes from a remote URL using browser `window.fetch()`.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;

    let resp_val = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("Network fetch failed for '{url}': {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())?;

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
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;

    let headers =
        web_sys::Headers::new().map_err(|e| format!("Failed to create headers: {e:?}"))?;
    let end = offset.saturating_add(length).saturating_sub(1);
    let range_val = format!("bytes={offset}-{end}");
    headers
        .set("Range", &range_val)
        .map_err(|e| format!("Failed to set Range header: {e:?}"))?;

    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(url, &init)
        .map_err(|e| format!("Failed to build request: {e:?}"))?;

    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Network range fetch failed for '{url}': {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())?;

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
    client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
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
