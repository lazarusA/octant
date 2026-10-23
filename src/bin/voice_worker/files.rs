//! A model's files for the worker, from the browser's cache or else straight
//! from their pinned URL (`octant::voice::whisper::models`): the download is
//! read as it arrives for its progress, checked against its SHA-256, and
//! only then cached, so the cache holds whole, checked files.

use js_sys::{Reflect, Uint8Array};
use octant::voice::js_text;
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
    let caches = scope.caches().map_err(js_text)?;
    let cache: Cache = wait(caches.open(CACHE)).await?.unchecked_into();
    let cached = wait(cache.match_with_str(&url)).await?;
    if let Some(response) = cached.dyn_ref::<Response>() {
        return bytes(response).await;
    }
    let response: Response = wait(scope.fetch_with_str(&url)).await?.unchecked_into();
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    let weights = file == models::WEIGHTS;
    let capacity = if weights {
        models::weights_bytes(size) as usize
    } else {
        0
    };
    let mut share = models::weights_progress(size, progress);
    let mut body = read(&response, capacity, |done| {
        if weights {
            share(done as u64);
        }
    })
    .await?;
    let got = models::hex(&Sha256::digest(&body));
    if got != sha256 {
        return Err(format!("{url}: checksum {got}, expected {sha256}"));
    }
    let stored = Response::new_with_opt_u8_array(Some(&mut body)).map_err(js_text)?;
    wait(cache.put_with_str(&url, &stored)).await?;
    Ok(body)
}

/// The body of `response` read as it arrives into room for `capacity` bytes
/// (the expected size, so a large body is never regrown), telling
/// `progress` the bytes so far.
async fn read(
    response: &Response,
    capacity: usize,
    mut progress: impl FnMut(usize),
) -> Result<Vec<u8>, String> {
    let Some(stream) = response.body() else {
        return bytes(response).await;
    };
    let reader: ReadableStreamDefaultReader = stream.get_reader().unchecked_into();
    let mut body = Vec::with_capacity(capacity);
    loop {
        let chunk = wait(reader.read()).await?;
        let done = Reflect::get(&chunk, &"done".into()).map_err(js_text)?;
        if done.as_bool() == Some(true) {
            return Ok(body);
        }
        let value: Uint8Array = Reflect::get(&chunk, &"value".into())
            .map_err(js_text)?
            .unchecked_into();
        let start = body.len();
        body.resize(start + value.length() as usize, 0);
        value.copy_to(&mut body[start..]);
        progress(body.len());
    }
}

async fn bytes(response: &Response) -> Result<Vec<u8>, String> {
    let buffer = wait(response.array_buffer().map_err(js_text)?).await?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

async fn wait(promise: js_sys::Promise) -> Result<JsValue, String> {
    JsFuture::from(promise).await.map_err(js_text)
}
