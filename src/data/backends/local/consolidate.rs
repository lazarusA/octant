//! Consolidated metadata for local Zarr folders that have none. The web's
//! Zarr reader lists nothing (a server cannot be listed), so it opens only
//! stores whose variables a consolidated document names; a local folder can
//! be listed, so the document is built from it: v2 stores are served a
//! `.zmetadata`, v3 stores a root `zarr.json` with `consolidated_metadata`.
//! Only group folders are listed; an array's folder holds its chunks.

use super::read::{Missing, blob_bytes, call, file_in, handle_dir, with_callbacks};
use super::registry::Source;
use super::{documents, path};
use js_sys::{Array, Function, Reflect};
use serde_json::{Map, Value};
use wasm_bindgen::{JsCast, JsValue};

/// The metadata documents a node's folder may hold.
const NODE_DOCS: [&str; 4] = ["zarr.json", ".zarray", ".zgroup", ".zattrs"];

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
    let document = documents::v2_document(metadata_docs(source).await);
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
    documents::consolidate_v3(&mut root, metadata_docs(source).await);
    serde_json::to_vec(&root).unwrap_or(bytes)
}

/// Every node's metadata documents by key, from the root down through its
/// groups.
async fn metadata_docs(source: &Source) -> Map<String, Value> {
    let mut docs = Map::new();
    let mut groups = vec![String::new()];
    while let Some(dir) = groups.pop() {
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        let mut group = false;
        for name in NODE_DOCS {
            let key = format!("{prefix}{name}");
            let Some(doc) = json_at(source, &key).await else {
                continue;
            };
            let v3_group = doc.get("node_type").and_then(Value::as_str) == Some("group");
            group |= name == ".zgroup" || (name == "zarr.json" && v3_group);
            docs.insert(key, doc);
            // A v3 node has no v2 documents beside its `zarr.json`.
            if name == "zarr.json" {
                break;
            }
        }
        if group {
            for folder in subfolders(source, &dir).await {
                groups.push(format!("{prefix}{folder}"));
            }
        }
    }
    docs
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
