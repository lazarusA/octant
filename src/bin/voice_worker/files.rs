//! A model's files for the worker, from the browser's cache or else straight
//! from their pinned URL (`octant::voice::whisper::models`): the download is
//! read as it arrives for its progress, checked against its SHA-256, and
//! only then cached, so the cache holds whole, checked files.

use js_sys::{Reflect, Uint8Array};
use octant::voice::stt::WhisperSize;
use octant::voice::whisper::models;
use sha2::{Digest, Sha256};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, ReadableStreamDefaultReader, Response, WorkerGlobalScope};

const CACHE: &str = "octant-whisper";

/// The bytes of `file` (name, SHA-256) of `size`; `progress` follows a
/// download as the share done.
pub async fn fetch(
    size: WhisperSize,
    (file, sha256): (&str, &str),
    progress: &dyn Fn(f32),
) -> Result<Vec<u8>, String> {
    let url = models::url(size, file);
    let scope: WorkerGlobalScope = js_sys::global().unchecked_into();
    let caches = scope.caches().map_err(text)?;
    let cache: Cache = wait(caches.open(CACHE)).await?.unchecked_into();
    let cached = wait(cache.match_with_str(&url)).await?;
    if let Some(response) = cached.dyn_ref::<Response>() {
        return bytes(response).await;
    }
    let response: Response = wait(scope.fetch_with_str(&url)).await?.unchecked_into();
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    let total = models::weights_bytes(size) as f32;
    let body = read(&response, |done| {
        if file == models::WEIGHTS {
            progress(done as f32 / total);
        }
    })
    .await?;
    let got: String = Sha256::digest(&body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if got != sha256 {
        return Err(format!("{url}: checksum {got}, expected {sha256}"));
    }
    let mut copy = body.clone();
    let stored = Response::new_with_opt_u8_array(Some(&mut copy)).map_err(text)?;
    wait(cache.put_with_str(&url, &stored)).await?;
    Ok(body)
}

/// The body of `response` read as it arrives, telling `progress` the bytes
/// so far.
async fn read(response: &Response, progress: impl Fn(usize)) -> Result<Vec<u8>, String> {
    let Some(stream) = response.body() else {
        return bytes(response).await;
    };
    let reader: ReadableStreamDefaultReader = stream.get_reader().unchecked_into();
    let mut body = Vec::new();
    loop {
        let chunk = wait(reader.read()).await?;
        let done = Reflect::get(&chunk, &"done".into()).map_err(text)?;
        if done.as_bool() == Some(true) {
            return Ok(body);
        }
        let value: Uint8Array = Reflect::get(&chunk, &"value".into())
            .map_err(text)?
            .unchecked_into();
        body.extend(value.to_vec());
        progress(body.len());
    }
}

async fn bytes(response: &Response) -> Result<Vec<u8>, String> {
    let buffer = wait(response.array_buffer().map_err(text)?).await?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

async fn wait(promise: js_sys::Promise) -> Result<JsValue, String> {
    JsFuture::from(promise).await.map_err(text)
}

fn text(value: JsValue) -> String {
    value
        .dyn_ref::<js_sys::Error>()
        .map(|e| String::from(e.message()))
        .or_else(|| value.as_string())
        .unwrap_or_else(|| format!("{value:?}"))
}
