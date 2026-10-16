//! Thread-safe WebAssembly GeoTIFF store cache and background loader registry.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use crate::data::DatasetMetadata;
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

use super::super::reader::WasmHttpTiffReader;
use super::super::store::GeoTiffBlockStore;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_GEOTIFF_STORES: std::cell::RefCell<HashMap<String, Arc<WasmGeoTiffBlockStore>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
static WASM_GEOTIFF_STORES_DESKTOP: OnceLock<Mutex<HashMap<String, Arc<WasmGeoTiffBlockStore>>>> =
    OnceLock::new();

/// WASM-compatible GeoTIFF/COG BlockStore backed by background HTTP range streaming.
pub struct WasmGeoTiffBlockStore {
    pub url: String,
    pub inner: Arc<RwLock<Option<GeoTiffBlockStore>>>,
    pub load_error: Arc<RwLock<Option<String>>>,
}

impl WasmGeoTiffBlockStore {
    #[cfg(target_arch = "wasm32")]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url.trim().to_string();
        WASM_GEOTIFF_STORES.with(|stores| {
            let mut map = stores.borrow_mut();
            if let Some(store) = map.get(&clean_url) {
                store.clone()
            } else {
                let new_store = Self::spawn_loader(&clean_url);
                map.insert(clean_url, new_store.clone());
                new_store
            }
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url.trim().to_string();
        let mutex = WASM_GEOTIFF_STORES_DESKTOP.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = mutex.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(store) = map.get(&clean_url) {
            store.clone()
        } else {
            let new_store = Self::spawn_loader(&clean_url);
            map.insert(clean_url, new_store.clone());
            new_store
        }
    }

    /// Drops the store of `url` from the registry (its dataset closed).
    pub fn release(url: &str) {
        let clean_url = url.trim();
        #[cfg(target_arch = "wasm32")]
        let _released = WASM_GEOTIFF_STORES.with(|stores| stores.borrow_mut().remove(clean_url));
        #[cfg(not(target_arch = "wasm32"))]
        let _released = WASM_GEOTIFF_STORES_DESKTOP.get().and_then(|stores| {
            stores
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(clean_url)
        });
    }

    fn spawn_loader(url: &str) -> Arc<Self> {
        let inner = Arc::new(RwLock::new(None));
        let load_error = Arc::new(RwLock::new(None));
        let store = Arc::new(Self {
            url: url.to_string(),
            inner: inner.clone(),
            load_error: load_error.clone(),
        });

        let target_url = url.to_string();
        let loader_fut = async move {
            let reader = Arc::new(WasmHttpTiffReader::new(&target_url));
            match GeoTiffBlockStore::from_reader(&target_url, reader).await {
                Ok(geo_store) => {
                    let mut guard = inner.write().unwrap_or_else(|p| p.into_inner());
                    *guard = Some(geo_store);
                }
                Err(e) => {
                    let mut guard = load_error.write().unwrap_or_else(|p| p.into_inner());
                    *guard = Some(e.to_string());
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(loader_fut);
        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::get_shared_tokio_rt().spawn(loader_fut);

        store
    }
}

impl BlockStore for WasmGeoTiffBlockStore {
    fn backend_name(&self) -> &str {
        "GeoTIFF (Web COG)"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let guard = self.inner.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref store) = *guard {
            store.variables()
        } else {
            Ok(Vec::new())
        }
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        let guard = self.inner.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref store) = *guard {
            return store.inspect();
        }
        drop(guard);

        let err_guard = self.load_error.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref e) = *err_guard {
            return Err(format!("Failed to load GeoTIFF at '{}': {e}", self.url).into());
        }

        Err(format!(
            "Metadata for '{}' is loading in the background...",
            self.url
        )
        .into())
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        let guard = self.inner.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref store) = *guard {
            store.fetch_block_with_progress(request, on_progress)
        } else {
            Err("GeoTIFF dataset is still loading metadata...".into())
        }
    }
}
