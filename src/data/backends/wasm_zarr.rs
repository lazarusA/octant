//! WebAssembly HTTP-backed Zarr store implementation.
//!
//! Provides in-memory caching and browser-native `fetch()` I/O
//! for streaming remote Zarr v2 and v3 datasets in web browsers.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use zarrs::array::ArraySubset;
use zarrs::group::Group;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadata;
use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{ReadableStorageTraits, StoreKey, WritableStorageTraits};

use super::zarr_block::fetch_block_with_progress;
use crate::data::block_request::BlockRequest;
use crate::data::block_store::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::metadata::{
    ParsedCfAttributes, open_or_instantiate_array_normalized, variable_info_from_array,
    variable_info_from_node_metadata,
};
use crate::utils::units::calculate_variable_size_bytes;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_STORES: std::cell::RefCell<HashMap<String, Arc<WasmZarrBlockStore>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
static WASM_STORES_DESKTOP: OnceLock<Mutex<HashMap<String, Arc<WasmZarrBlockStore>>>> =
    OnceLock::new();

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

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

/// WASM-compatible Zarr BlockStore backed by in-memory metadata and on-demand chunk fetching.
pub struct WasmZarrBlockStore {
    pub base_url: String,
    pub memory_store: ReadableWritableListableStorage,
    pub metadata: RwLock<Option<DatasetMetadata>>,
    pub is_local_notice: bool,
}

impl WasmZarrBlockStore {
    #[cfg(target_arch = "wasm32")]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url.trim_end_matches('/').to_string();
        WASM_STORES.with(|stores| {
            let mut map = stores.borrow_mut();
            if let Some(store) = map.get(&clean_url) {
                store.clone()
            } else {
                let new_store = Arc::new(Self {
                    base_url: clean_url.clone(),
                    memory_store: Arc::new(MemoryStore::new()),
                    metadata: RwLock::new(None),
                    is_local_notice: false,
                });
                map.insert(clean_url, new_store.clone());
                new_store
            }
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url.trim_end_matches('/').to_string();
        let stores = WASM_STORES_DESKTOP.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = stores.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(store) = guard.get(&clean_url) {
            store.clone()
        } else {
            let new_store = Arc::new(Self {
                base_url: clean_url.clone(),
                memory_store: Arc::new(MemoryStore::new()),
                metadata: RwLock::new(None),
                is_local_notice: false,
            });
            guard.insert(clean_url, new_store.clone());
            new_store
        }
    }

    pub fn open_local_notice(path: &str) -> Arc<Self> {
        Arc::new(Self {
            base_url: path.to_string(),
            memory_store: Arc::new(MemoryStore::new()),
            metadata: RwLock::new(None),
            is_local_notice: true,
        })
    }

    /// Stores a raw byte slice into the memory store under `key`.
    pub fn insert_key_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), BlockStoreError> {
        let clean_key = key.trim_start_matches('/');
        let store_key = StoreKey::new(clean_key).map_err(|e| format!("{e}"))?;
        let bytes_vec = zarrs::storage::Bytes::from(bytes.to_vec());
        self.memory_store
            .set(&store_key, bytes_vec)
            .map_err(|e| format!("{e}").into())
    }

    /// Checks if a key already exists in the memory store.
    pub fn has_key(&self, key: &str) -> bool {
        let clean_key = key.trim_start_matches('/');
        if let Ok(store_key) = StoreKey::new(clean_key) {
            self.memory_store
                .get(&store_key)
                .map(|b| b.is_some())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Computes and fetches all required chunk files for an array slice into memory.
    pub async fn preload_chunks_for_subset(
        &self,
        var_name: &str,
        subset: &ArraySubset,
        mut on_progress: ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let clean_var = var_name.trim_matches('/');
        let var_path = format!("/{clean_var}");

        let array = open_or_instantiate_array_normalized(self.memory_store.clone(), &var_path)
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("blosc") {
                    "Compressed with Blosc (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
                } else if err_str.contains("zstd") {
                    "Compressed with Zstd (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
                } else {
                    format!("Failed to open array '{clean_var}': {err_str}")
                }
            })?;

        let rank = array.shape().len();
        let zero_idx = vec![0u64; rank];

        let chunk_dims = array
            .chunk_shape(&zero_idx)
            .map_err(|e| format!("Failed to get chunk shape: {e}"))?;

        // Calculate chunk index ranges per dimension
        let mut chunk_ranges = Vec::with_capacity(rank);
        for (i, dim_len) in chunk_dims.iter().enumerate() {
            let c_len = dim_len.get();
            let sel_start = subset.start()[i];
            let sel_shape = subset.shape()[i];
            let sel_end = sel_start + sel_shape;

            let c_start = sel_start / c_len;
            let c_end = (sel_end.saturating_sub(1) / c_len) + 1;
            chunk_ranges.push(c_start..c_end);
        }

        // Iterate over Cartesian product of chunk indices
        let mut current = chunk_ranges.iter().map(|r| r.start).collect::<Vec<u64>>();
        loop {
            let chunk_rel_key = array.chunk_key(&current);
            let store_key_str = if clean_var.is_empty() {
                chunk_rel_key.as_str().to_string()
            } else {
                format!("{clean_var}/{}", chunk_rel_key.as_str())
            };

            if !self.has_key(&store_key_str) {
                let chunk_url = format!("{}/{}", self.base_url, store_key_str);
                log::debug!("[WASM Zarr] Fetching chunk: {chunk_url}");

                match fetch_url_bytes(&chunk_url).await {
                    Ok(chunk_bytes) => {
                        let bytes_len = chunk_bytes.len() as u64;
                        self.insert_key_bytes(&store_key_str, &chunk_bytes)?;

                        if let Some(ref mut cb) = on_progress {
                            cb(bytes_len);
                        }
                    }
                    Err(err) if err.contains("HTTP 404") => {
                        log::debug!(
                            "[WASM Zarr] Chunk not found (404 / sparse chunk): {chunk_url}"
                        );
                        // In Zarr specification, missing chunks are treated as fill value.
                    }
                    Err(err) => {
                        return Err(format!("Failed to fetch chunk '{chunk_url}': {err}").into());
                    }
                }
            }

            // Advance chunk indices
            let mut carry = true;
            for i in (0..rank).rev() {
                current[i] += 1;
                if current[i] < chunk_ranges[i].end {
                    carry = false;
                    break;
                }
                current[i] = chunk_ranges[i].start;
            }
            if carry {
                break;
            }
        }

        Ok(())
    }
}

impl BlockStore for WasmZarrBlockStore {
    fn backend_name(&self) -> &str {
        if self.is_local_notice {
            "Local Zarr (Browser Sandbox Notice)"
        } else {
            "Zarr (Web HTTP)"
        }
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let guard = self.metadata.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref meta) = *guard {
            Ok(meta.variables.iter().map(|v| v.name.clone()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        if self.is_local_notice {
            return Err("Direct local file paths cannot be read in a browser due to web sandbox security.\n\nTo view local Zarr files in the browser:\n1. Serve your directory with a local HTTP server: `npx serve` or `python3 -m http.server`\n2. Enter the URL: `http://localhost:8000/my_dataset.zarr`\n\nOr run the native desktop version of Octant (`cargo run --release`).".into());
        }

        let guard = self.metadata.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref meta) = *guard {
            return Ok(meta.clone());
        }
        drop(guard);

        Err(format!(
            "Metadata for '{}' is loading in the background...",
            self.base_url
        )
        .into())
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        fetch_block_with_progress(
            self.memory_store.clone(),
            &self.base_url,
            request,
            on_progress,
        )
    }
}

/// Asynchronously inspects a remote Zarr URL in the browser and returns `DatasetMetadata`.
pub async fn inspect_wasm_remote_zarr(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim_end_matches('/');
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }

    let store = WasmZarrBlockStore::get_or_create(clean_url);

    // 1. Try fetching Zarr v2 consolidated `.zmetadata`
    let zmetadata_url = format!("{clean_url}/.zmetadata");
    if let Ok(zmetadata_bytes) = fetch_url_bytes(&zmetadata_url).await
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&zmetadata_bytes)
    {
        let _ = store.insert_key_bytes(".zmetadata", &zmetadata_bytes);

        if let Some(metadata_map) = val.get("metadata").and_then(|m| m.as_object()) {
            for (k, v) in metadata_map {
                if let Ok(json_str) = serde_json::to_string(v) {
                    let _ = store.insert_key_bytes(k, json_str.as_bytes());
                }
            }
        }

        let mut variables = Vec::new();
        if let Some(metadata_obj) = val.get("metadata").and_then(|m| m.as_object()) {
            for (key, val) in metadata_obj {
                if key.ends_with("/.zarray") || key == ".zarray" || key.ends_with("/zarr.json") {
                    let var_name = key
                        .trim_end_matches("/.zarray")
                        .trim_end_matches("/zarr.json")
                        .trim_start_matches('/')
                        .to_string();
                    let var_name = if var_name.is_empty() {
                        "data".to_string()
                    } else {
                        var_name
                    };

                    let shape: Vec<u64> = val
                        .get("shape")
                        .and_then(|s| s.as_array())
                        .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                        .unwrap_or_default();

                    if shape.is_empty() {
                        continue;
                    }

                    let data_type = val
                        .get("dtype")
                        .or_else(|| val.get("data_type"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("float32")
                        .to_string();

                    let chunk_shape: Vec<u64> = val
                        .get("chunks")
                        .or_else(|| {
                            val.get("chunk_grid")
                                .and_then(|cg| cg.get("configuration"))
                                .and_then(|c| c.get("chunk_shape"))
                        })
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                        .unwrap_or_else(|| shape.clone());

                    let attrs_key = if key == ".zarray" {
                        ".zattrs".to_string()
                    } else {
                        format!("{var_name}/.zattrs")
                    };

                    let cf_attrs = metadata_obj
                        .get(&attrs_key)
                        .and_then(|a| a.as_object())
                        .map(ParsedCfAttributes::from_json_map)
                        .unwrap_or_default();

                    let dimension_names = cf_attrs.resolve_dimension_names(None, shape.len());
                    let file_size = calculate_variable_size_bytes(&shape, &data_type);

                    variables.push(VariableInfo {
                        name: var_name,
                        data_type,
                        shape,
                        dimension_names,
                        chunk_shape,
                        file_size,
                        units: cf_attrs.units,
                        long_name: cf_attrs.long_name,
                        time_coverage_start: cf_attrs.time_coverage_start,
                        time_coverage_end: cf_attrs.time_coverage_end,
                        temporal_resolution: cf_attrs.temporal_resolution,
                        attributes: cf_attrs.attributes,
                    });
                }
            }
        }

        if !variables.is_empty() {
            let dataset_name = clean_url
                .split('/')
                .next_back()
                .unwrap_or(clean_url)
                .to_string();

            let dataset_metadata = DatasetMetadata {
                name: dataset_name,
                store_type: "zarr".to_string(),
                variables,
                dimension_coordinates: HashMap::new(),
            };

            let mut guard = store.metadata.write().unwrap_or_else(|p| p.into_inner());
            *guard = Some(dataset_metadata.clone());
            return Ok(dataset_metadata);
        }
    }

    // 2. Try Zarr v3 `zarr.json` root
    let root_v3_url = format!("{clean_url}/zarr.json");
    if let Ok(v3_bytes) = fetch_url_bytes(&root_v3_url).await {
        let _ = store.insert_key_bytes("zarr.json", &v3_bytes);

        if let Ok(group) = Group::open(store.memory_store.clone(), "/") {
            let mut variables = Vec::new();
            if let Some(ConsolidatedMetadata { metadata, .. }) = group.consolidated_metadata() {
                for (name, node_meta) in metadata {
                    if let Some(var_info) = variable_info_from_node_metadata(&name, &node_meta) {
                        variables.push(var_info);
                    }
                }
            }

            if !variables.is_empty() {
                let dataset_name = clean_url
                    .split('/')
                    .next_back()
                    .unwrap_or(clean_url)
                    .to_string();
                let dataset_metadata = DatasetMetadata {
                    name: dataset_name,
                    store_type: "zarr".to_string(),
                    variables,
                    dimension_coordinates: HashMap::new(),
                };
                let mut guard = store.metadata.write().unwrap_or_else(|p| p.into_inner());
                *guard = Some(dataset_metadata.clone());
                return Ok(dataset_metadata);
            }
        }

        // Single root array case
        if let Ok(arr) = open_or_instantiate_array_normalized(store.memory_store.clone(), "/")
            && let Some(var_info) = variable_info_from_array(&arr, "data")
        {
            let dataset_name = clean_url
                .split('/')
                .next_back()
                .unwrap_or(clean_url)
                .to_string();
            let dataset_metadata = DatasetMetadata {
                name: dataset_name,
                store_type: "zarr".to_string(),
                variables: vec![var_info],
                dimension_coordinates: HashMap::new(),
            };
            let mut guard = store.metadata.write().unwrap_or_else(|p| p.into_inner());
            *guard = Some(dataset_metadata.clone());
            return Ok(dataset_metadata);
        }
    }

    Err(format!(
        "Failed to inspect Zarr metadata at '{clean_url}'. Ensure the server allows CORS (Access-Control-Allow-Origin) and contains '.zmetadata' or 'zarr.json'."
    ))
}

/// Asynchronously loads one block on WASM, preloading required chunk bytes over HTTP if needed.
pub async fn load_one_wasm_with_progress(
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    let source_uri = &request.store.source().uri;

    // If it's a procedural dataset or non-HTTP store, fetch directly
    if source_uri.starts_with("procedural://") || source_uri.starts_with("test://") {
        return request
            .store
            .fetch_with_progress(&request.slice, on_progress);
    }

    let clean_url = source_uri.trim_end_matches('/');
    let store = WasmZarrBlockStore::get_or_create(clean_url);

    let shape = request
        .store
        .inspect()
        .map(|meta| {
            meta.variables
                .iter()
                .find(|v| v.name == request.slice.variable)
                .map(|v| v.shape.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let rank = request.slice.selections.len();
    let mut ranges = Vec::with_capacity(rank);

    for (i, sel) in request.slice.selections.iter().enumerate() {
        let dim_len = shape.get(i).copied().unwrap_or(1000) as usize;
        let (start, end) = match *sel {
            crate::data::slice_request::DimensionSelection::Index(idx) => {
                (idx, idx.saturating_add(1))
            }
            crate::data::slice_request::DimensionSelection::Range { start, end } => (start, end),
        };
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start + 1).min(dim_len);
        ranges.push(start as u64..end as u64);
    }

    let subset = ArraySubset::new_with_ranges(&ranges);
    log::info!(
        "[WASM Zarr] Preloading chunks for '{}', subset: {:?}",
        request.slice.variable,
        subset.to_ranges()
    );

    // Asynchronously download any missing chunks for this slice
    if let Err(e) = store
        .preload_chunks_for_subset(&request.slice.variable, &subset, on_progress)
        .await
    {
        let err_str = e.to_string();
        let user_msg = if err_str.contains("blosc") {
            "Compressed with Blosc (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
        } else if err_str.contains("zstd") {
            "Compressed with Zstd (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
        } else {
            err_str
        };
        log::error!(
            "[WASM Zarr] Failed downloading chunks for '{}': {user_msg}",
            request.slice.variable
        );
        return Err(user_msg.into());
    }

    // Now decode the slice synchronously from in-memory chunks
    match store.fetch_block_with_progress(&request.slice, None) {
        Ok(block) => {
            log::info!(
                "[WASM Zarr] Successfully decoded block for '{}' ({} values, shape: {:?})",
                request.slice.variable,
                block.values.len(),
                block.shape
            );
            Ok(block)
        }
        Err(e) => {
            let err_str = e.to_string();
            let user_msg = if err_str.contains("blosc") {
                "Compressed with Blosc (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
            } else if err_str.contains("zstd") {
                "Compressed with Zstd (supported in desktop build). For browser streaming, export with uncompressed chunks.".to_string()
            } else {
                err_str
            };
            log::error!(
                "[WASM Zarr] Failed decoding block for '{}': {user_msg}",
                request.slice.variable
            );
            Err(user_msg.into())
        }
    }
}
