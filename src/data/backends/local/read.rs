//! Reading local sources: a key's file found in its source, then read by
//! range. A key that is not there answers `HTTP 404 ...`, as a remote store
//! does, so `http::is_not_found` and `is_missing` read it the same way.

use super::path;
use super::registry::{Source, source_of};
use crate::data::DataSourceKind;
use js_sys::{Array, Function, Promise, Reflect, Uint8Array};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

thread_local! {
    /// The consolidated metadata built for local Zarr folders, by URL: built
    /// once, however many readers ask (the page and each data worker).
    static BUILT: RefCell<HashMap<String, Vec<u8>>> = RefCell::default();
}

/// The bytes of the file at local `url`. A Zarr folder's consolidated
/// metadata, which the web's Zarr reader needs, is built from its folders
/// when the store has none (`consolidate.rs`).
pub async fn read_all(url: &str) -> Result<Vec<u8>, String> {
    if let Some(built) = BUILT.with_borrow(|b| b.get(url).cloned()) {
        return Ok(built);
    }
    let (source, key) = source_of(url).ok_or_else(|| not_found(url))?;
    let built = match file_in(&source, key).await {
        Err(Missing::NotFound) if key == ".zmetadata" => {
            let built = super::consolidate::zmetadata(&source).await;
            built.ok_or_else(|| not_found(url))?
        }
        found => {
            let file = found.map_err(|missing| failure(url, missing))?;
            let bytes = blob_bytes(&file).await?;
            if key != "zarr.json" {
                return Ok(bytes);
            }
            super::consolidate::v3_root(&source, bytes).await
        }
    };
    // Kept only while its source is: one released during the build is not.
    if source_of(url).is_some() {
        BUILT.with_borrow_mut(|b| b.insert(url.to_owned(), built.clone()));
    }
    Ok(built)
}

/// Forgets the documents built for the local source `id` (released).
pub(super) fn forget_built(id: u32) {
    BUILT.with_borrow_mut(|b| b.retain(|url, _| path::parse(url).map(|(i, _)| i) != Some(id)));
}

/// `length` bytes of the file at local `url` from `offset` (fewer at its end).
pub async fn read_range(url: &str, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    let file = file_at(url).await?;
    let end = offset.saturating_add(length);
    let part = file
        .slice_with_f64_and_f64(offset as f64, end as f64)
        .map_err(|e| format!("Failed to slice '{url}': {e:?}"))?;
    blob_bytes(&part).await
}

/// The size in bytes of the file at local `url`.
pub async fn size(url: &str) -> Result<u64, String> {
    Ok(file_at(url).await?.size() as u64)
}

/// The store kind `source`, named `name`, opens as: a file by its extension
/// (GeoTIFF only), a folder as Icechunk when it holds Icechunk's `repo` or
/// main branch reference, else Zarr.
pub(super) async fn kind_in(source: &Source, name: &str) -> Option<DataSourceKind> {
    if let Source::File(_) = source {
        use crate::app::StoreKind;
        return match crate::utils::infer_store_kind_from_target(name) {
            Ok(kind) if kind.is_geotiff() => Some(DataSourceKind::RemoteGeoTiff),
            Ok(StoreKind::LocalNetCdf) => Some(DataSourceKind::NetCdf),
            Ok(StoreKind::Grib) => Some(DataSourceKind::Grib),
            _ => None,
        };
    }
    let marks = ["repo", "refs/branch.main/ref.json"];
    for key in marks {
        if file_in(source, key).await.is_ok() {
            return Some(DataSourceKind::RemoteIcechunk);
        }
    }
    Some(DataSourceKind::RemoteZarr)
}

async fn file_at(url: &str) -> Result<web_sys::File, String> {
    let (source, key) = source_of(url).ok_or_else(|| not_found(url))?;
    file_in(&source, key)
        .await
        .map_err(|missing| failure(url, missing))
}

fn failure(url: &str, missing: Missing) -> String {
    match missing {
        Missing::NotFound => not_found(url),
        Missing::Failed(e) => format!("Failed to read '{url}': {e}"),
    }
}

pub(super) enum Missing {
    NotFound,
    Failed(String),
}

fn not_found(url: &str) -> String {
    format!("HTTP 404 reading '{url}'")
}

/// The file at `key` in `source`.
pub(super) async fn file_in(source: &Source, key: &str) -> Result<web_sys::File, Missing> {
    if let Source::File(file) = source {
        return key
            .is_empty()
            .then(|| file.clone())
            .ok_or(Missing::NotFound);
    }
    let parts = path::segments(key).ok_or(Missing::NotFound)?;
    match source {
        Source::Listed(files) => files.get(key).cloned().ok_or(Missing::NotFound),
        Source::Handle(dir) => handle_file(dir, &parts).await.map_err(missing),
        Source::Entry(dir) => entry_file(dir, key).await.map_err(missing),
        Source::File(_) => Err(Missing::NotFound),
    }
}

/// A lookup error as `Missing`: what the File System APIs throw for absent
/// names (or a folder where a file is asked) is `NotFound`.
fn missing(err: JsValue) -> Missing {
    let name = Reflect::get(&err, &"name".into())
        .ok()
        .and_then(|n| n.as_string());
    match name.as_deref() {
        Some("NotFoundError" | "TypeMismatchError" | "TypeError") => Missing::NotFound,
        _ => Missing::Failed(format!("{err:?}")),
    }
}

/// `parts` under a `FileSystemDirectoryHandle`, a handle per folder.
async fn handle_file(dir: &JsValue, parts: &[&str]) -> Result<web_sys::File, JsValue> {
    let Some((name, folders)) = parts.split_last() else {
        return Err(JsValue::from_str("empty key"));
    };
    let dir = handle_dir(dir, folders).await?;
    let handle = call(&dir, "getFileHandle", &Array::of1(&(*name).into())).await?;
    call(&handle, "getFile", &Array::new()).await?.dyn_into()
}

/// The folder at `parts` under a `FileSystemDirectoryHandle`.
pub(super) async fn handle_dir(dir: &JsValue, parts: &[&str]) -> Result<JsValue, JsValue> {
    let mut dir = dir.clone();
    for folder in parts {
        dir = call(&dir, "getDirectoryHandle", &Array::of1(&(*folder).into())).await?;
    }
    Ok(dir)
}

/// `key` under a dropped `FileSystemDirectoryEntry`, whose callback API
/// takes a relative path.
async fn entry_file(dir: &JsValue, key: &str) -> Result<web_sys::File, JsValue> {
    let options = js_sys::Object::new();
    let entry = with_callbacks(dir, "getFile", &Array::of2(&key.into(), &options)).await?;
    entry_as_file(&entry).await
}

/// The file of a `FileSystemFileEntry`.
pub(super) async fn entry_as_file(entry: &JsValue) -> Result<web_sys::File, JsValue> {
    with_callbacks(entry, "file", &Array::new())
        .await?
        .dyn_into()
}

/// Calls `target.method(...args)` and awaits the promise it returns.
pub(super) async fn call(target: &JsValue, method: &str, args: &Array) -> Result<JsValue, JsValue> {
    let function: Function = Reflect::get(target, &method.into())?.dyn_into()?;
    let promise: Promise = function.apply(target, args)?.dyn_into()?;
    JsFuture::from(promise).await
}

/// Calls `target.method(...args, ok, err)`, the callback style of the File
/// and Directory Entries API, and awaits the callback it calls.
pub(super) async fn with_callbacks(
    target: &JsValue,
    method: &str,
    args: &Array,
) -> Result<JsValue, JsValue> {
    let promise = Promise::new(&mut |resolve, reject| {
        let called = Reflect::get(target, &method.into())
            .and_then(|f| f.dyn_into::<Function>())
            .and_then(|f| {
                let all = args.concat(&Array::of2(&resolve, &reject));
                f.apply(target, &all)
            });
        if let Err(e) = called {
            let _ = reject.call1(&JsValue::NULL, &e);
        }
    });
    JsFuture::from(promise).await
}

pub(super) async fn blob_bytes(blob: &web_sys::Blob) -> Result<Vec<u8>, String> {
    let buffer = JsFuture::from(blob.array_buffer())
        .await
        .map_err(|e| format!("Failed to read a local file: {e:?}"))?;
    Ok(Uint8Array::new(&buffer).to_vec())
}
