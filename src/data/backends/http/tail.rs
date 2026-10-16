//! The tail of a remote object: a shard's index sits at its end, read
//! with a suffix range and placed through the object's total size.

/// The total size in a `Content-Range` header (`bytes 10-19/2000`).
pub fn content_range_total(header: &str) -> Option<u64> {
    header.rsplit_once('/')?.1.trim().parse().ok()
}

/// The last `length` bytes of `body`, read whole by a server that ignored
/// the range: `(tail, total size)`.
fn tail_of(body: Vec<u8>, length: u64) -> (Vec<u8>, u64) {
    let total = body.len() as u64;
    let start = total.saturating_sub(length) as usize;
    (body[start..].to_vec(), total)
}

/// The last `length` bytes of the object at `url` and its total size (from
/// `Content-Range`, which the server must expose to the page): a shard's
/// index at its end. Fails when the size can't be read.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url_tail(url: &str, length: u64) -> Result<(Vec<u8>, u64), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;
    let headers =
        web_sys::Headers::new().map_err(|e| format!("Failed to create headers: {e:?}"))?;
    headers
        .set("Range", &format!("bytes=-{length}"))
        .map_err(|e| format!("Failed to set Range header: {e:?}"))?;
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_headers(&headers);
    let request = web_sys::Request::new_with_str_and_init(url, &init)
        .map_err(|e| format!("Failed to build request: {e:?}"))?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Network range fetch failed for '{url}': {e:?}"))?
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())?;
    let status = resp.status();
    if status != 200 && status != 206 {
        return Err(format!("HTTP {status} fetching the tail of '{url}'"));
    }
    let content_range = resp.headers().get("Content-Range").ok().flatten();
    let buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("Failed to read array buffer: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("Failed to resolve array buffer: {e:?}"))?;
    let body = js_sys::Uint8Array::new(&buffer).to_vec();
    if status == 200 {
        return Ok(tail_of(body, length));
    }
    let total = content_range
        .as_deref()
        .and_then(content_range_total)
        .ok_or_else(|| format!("No readable Content-Range for '{url}'"))?;
    Ok((body, total))
}

/// The last `length` bytes of the object at `url` and its total size, on
/// desktop targets via `reqwest`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_url_tail(url: &str, length: u64) -> Result<(Vec<u8>, u64), String> {
    let resp = super::fetch::get_http_client()
        .get(url)
        .header("Range", format!("bytes=-{length}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if status != 200 && status != 206 {
        return Err(format!("HTTP {status} fetching the tail of '{url}'"));
    }
    let content_range = resp
        .headers()
        .get("Content-Range")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
    if status == 200 {
        return Ok(tail_of(body, length));
    }
    let total = content_range
        .as_deref()
        .and_then(content_range_total)
        .ok_or_else(|| format!("No readable Content-Range for '{url}'"))?;
    Ok((body, total))
}

#[cfg(test)]
mod tests {
    #[test]
    fn content_ranges_give_the_total_size() {
        assert_eq!(
            super::content_range_total("bytes 9990-9999/10000"),
            Some(10000)
        );
        assert_eq!(super::content_range_total("bytes 0-9/*"), None);
        assert_eq!(super::tail_of((0..10).collect(), 3), (vec![7, 8, 9], 10));
    }
}
