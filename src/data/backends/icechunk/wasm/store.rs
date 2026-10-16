//! BlockStore implementation and caching for WebAssembly Icechunk data streaming.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use icechunk_format::manifest::ManifestRef;

use super::manifest_cache::ManifestCache;

use crate::data::DatasetMetadata;
use crate::data::backends::zarr::WasmZarrBlockStore;
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_ICECHUNK_STORES: std::cell::RefCell<HashMap<String, Arc<WasmIcechunkBlockStore>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
static WASM_ICECHUNK_STORES_DESKTOP: OnceLock<Mutex<HashMap<String, Arc<WasmIcechunkBlockStore>>>> =
    OnceLock::new();

/// Manifest references and node ID recorded for a specific Icechunk array.
pub struct ArrayManifestInfo {
    pub node_id: icechunk_format::ObjectId<8, icechunk_format::NodeTag>,
    pub manifests: Vec<ManifestRef>,
}

/// WASM-compatible Icechunk BlockStore backed by in-memory metadata and on-demand chunk fetching.
pub struct WasmIcechunkBlockStore {
    pub base_url: String,
    pub inner: Arc<WasmZarrBlockStore>,
    pub array_manifests: RwLock<HashMap<String, ArrayManifestInfo>>,
    pub cached_manifests: Arc<ManifestCache>,
}

impl WasmIcechunkBlockStore {
    #[cfg(target_arch = "wasm32")]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url
            .trim_start_matches("icechunk+")
            .trim_end_matches('/')
            .to_string();
        WASM_ICECHUNK_STORES.with(|stores| {
            let mut map = stores.borrow_mut();
            if let Some(store) = map.get(&clean_url) {
                store.clone()
            } else {
                let inner = WasmZarrBlockStore::get_or_create(&clean_url);
                let new_store = Arc::new(Self {
                    base_url: clean_url.clone(),
                    inner,
                    array_manifests: RwLock::new(HashMap::new()),
                    cached_manifests: ManifestCache::new(),
                });
                map.insert(clean_url, new_store.clone());
                new_store
            }
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url
            .trim_start_matches("icechunk+")
            .trim_end_matches('/')
            .to_string();
        let stores = WASM_ICECHUNK_STORES_DESKTOP.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = stores.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(store) = guard.get(&clean_url) {
            store.clone()
        } else {
            let inner = WasmZarrBlockStore::get_or_create(&clean_url);
            let new_store = Arc::new(Self {
                base_url: clean_url.clone(),
                inner,
                array_manifests: RwLock::new(HashMap::new()),
                cached_manifests: ManifestCache::new(),
            });
            guard.insert(clean_url, new_store.clone());
            new_store
        }
    }
}

impl BlockStore for WasmIcechunkBlockStore {
    fn backend_name(&self) -> &str {
        "Icechunk (Web HTTP)"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.inner.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block(request)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.inner.fetch_block_with_progress(request, on_progress)
    }

    fn fetch_blocks(
        &self,
        requests: &[SliceRequest],
    ) -> Result<crate::data::blocks::BlockResult, BlockStoreError> {
        self.inner.fetch_blocks(requests)
    }
}
