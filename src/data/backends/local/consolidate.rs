//! Consolidated metadata for local Zarr folders that have none. The web's
//! Zarr reader lists nothing (a server cannot be listed), so it opens only
//! stores whose variables a consolidated document names; a local folder can
//! be listed, so the document is built from it: v2 stores are served a
//! `.zmetadata`, v3 stores a root `zarr.json` with `consolidated_metadata`.
//! Only group folders are listed; an array's folder holds its chunks.

use super::read::{Missing, blob_bytes, call, file_in, handle_dir, with_callbacks};
use super::registry::Source;
use super::{documents, path};
use futures::{StreamExt, stream};
use js_sys::{Array, Function, Reflect};
use serde_json::{Map, Value};
use wasm_bindgen::{JsCast, JsValue};

/// The metadata documents a v3 node's folder holds.
const V3_DOCS: &[&str] = &["zarr.json"];
/// The metadata documents a v2 node's folder may hold.
const V2_DOCS: &[&str] = &[".zarray", ".zgroup", ".zattrs"];
/// Files read, and folders listed, at the same time while walking.
const CONCURRENT_READS: usize = 32;

/// A v2 `.zmetadata` of the store in `source`; none when it is no v2 store,
/// or an OME-Zarr one (its discovery reads it by known paths, so a large
/// plate is never walked).
pub(super) async fn zmetadata(source: &Source) -> Option<Vec<u8>> {
    if json_at(source, ".zgroup").await.is_none() && json_at(source, ".zarray").await.is_none() {
        return None;
    }
    if documents::is_ome_root(json_at(source, ".zattrs").await.as_ref()) {
        return None;
    }
    let document = documents::v2_document(metadata_docs(source, V2_DOCS).await);
    serde_json::to_vec(&document).ok()
}

/// The root `zarr.json` read from `source`, with the store's nodes added as
/// its consolidated metadata when it needs them
/// (`documents::v3_needs_consolidation`).
pub(super) async fn v3_root(source: &Source, bytes: Vec<u8>) -> Vec<u8> {
    let Ok(mut root) = serde_json::from_slice::<Value>(&bytes) else {
        return bytes;
    };
    if !documents::v3_needs_consolidation(&root) {
        return bytes;
    }
    documents::consolidate_v3(&mut root, metadata_docs(source, V3_DOCS).await);
    serde_json::to_vec(&root).unwrap_or(bytes)
}

/// Every node's metadata documents (`names`, the store's format's) by key,
/// from the root down through its groups, a level of folders at a time.
async fn metadata_docs(source: &Source, names: &[&str]) -> Map<String, Value> {
    let mut docs = Map::new();
    let mut level = vec![String::new()];
    while !level.is_empty() {
        let keys = level
            .iter()
            .flat_map(|dir| names.iter().map(move |n| key_in(dir, n)));
        let found: Vec<(String, Option<Value>)> = stream::iter(keys)
            .map(|key| async move {
                let doc = json_at(source, &key).await;
                (key, doc)
            })
            .buffered(CONCURRENT_READS)
            .collect()
            .await;
        let groups: Vec<String> = found
            .iter()
            .filter_map(|(key, doc)| documents::group_of(key, doc.as_ref()?))
            .collect();
        docs.extend(found.into_iter().filter_map(|(key, doc)| Some((key, doc?))));
        level = stream::iter(groups)
            .map(|dir| async move {
                let names = subfolders(source, &dir).await;
                names
                    .into_iter()
                    .map(|n| key_in(&dir, &n))
                    .collect::<Vec<_>>()
            })
            .buffered(CONCURRENT_READS)
            .concat()
            .await;
    }
    docs
}

/// `name` in folder `dir` (`""` the root).
fn key_in(dir: &str, name: &str) -> String {
    match dir {
        "" => name.to_owned(),
        _ => format!("{dir}/{name}"),
    }
}

/// The JSON document at `key`, none when absent or unreadable.
async fn json_at(source: &Source, key: &str) -> Option<Value> {
    let file = match file_in(source, key).await {
        Ok(file) => file,
        Err(Missing::NotFound) => return None,
        Err(Missing::Failed(e)) => {
            log::warn!("Could not read local '{key}': {e}");
            return None;
        }
    };
    serde_json::from_slice(&blob_bytes(&file).await.ok()?).ok()
}

/// The names of the folders directly under folder `dir` (`""` the root).
async fn subfolders(source: &Source, dir: &str) -> Vec<String> {
    let listed = match source {
        Source::Listed(files) => {
            let keys = files.keys().map(String::as_str);
            return path::subfolders(keys, dir)
                .into_iter()
                .map(str::to_owned)
                .collect();
        }
        Source::Handle(root) => handle_subfolders(root, dir).await,
        Source::Entry(root) => entry_subfolders(root, dir).await,
        Source::File(_) => return Vec::new(),
    };
    listed.unwrap_or_else(|e| {
        log::warn!("Could not list local folder '{dir}': {e:?}");
        Vec::new()
    })
}

/// Subfolders through a `FileSystemDirectoryHandle`'s `values()` iterator.
async fn handle_subfolders(root: &JsValue, dir: &str) -> Result<Vec<String>, JsValue> {
    let parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    let folder = handle_dir(root, &parts).await?;
    let values: Function = Reflect::get(&folder, &"values".into())?.dyn_into()?;
    let entries = values.call0(&folder)?;
    let mut names = Vec::new();
    loop {
        let step = call(&entries, "next", &Array::new()).await?;
        if Reflect::get(&step, &"done".into())?.as_bool() == Some(true) {
            return Ok(names);
        }
        let handle = Reflect::get(&step, &"value".into())?;
        let kind = Reflect::get(&handle, &"kind".into())?.as_string();
        if kind.as_deref() == Some("directory")
            && let Some(name) = Reflect::get(&handle, &"name".into())?.as_string()
        {
            names.push(name);
        }
    }
}

/// Subfolders through a dropped `FileSystemDirectoryEntry`'s reader, which
/// hands entries over in batches until an empty one.
async fn entry_subfolders(root: &JsValue, dir: &str) -> Result<Vec<String>, JsValue> {
    let folder = match dir {
        "" => root.clone(),
        _ => {
            let args = Array::of2(&dir.into(), &js_sys::Object::new());
            with_callbacks(root, "getDirectory", &args).await?
        }
    };
    let create: Function = Reflect::get(&folder, &"createReader".into())?.dyn_into()?;
    let reader = create.call0(&folder)?;
    let mut names = Vec::new();
    loop {
        let batch: Array = with_callbacks(&reader, "readEntries", &Array::new())
            .await?
            .dyn_into()?;
        if batch.length() == 0 {
            return Ok(names);
        }
        for entry in batch.iter() {
            let is_folder = Reflect::get(&entry, &"isDirectory".into())?.as_bool();
            if is_folder == Some(true)
                && let Some(name) = Reflect::get(&entry, &"name".into())?.as_string()
            {
                names.push(name);
            }
        }
    }
}
