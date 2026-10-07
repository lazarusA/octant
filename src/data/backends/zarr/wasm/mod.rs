//! WebAssembly HTTP-backed Zarr store.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use crate::data::DatasetMetadata;
use crate::data::blocks::BlockStoreError;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{ReadableWritableListableStorage, StoreKey, WritableStorageTraits};

pub mod coord_paths;
pub mod inspect;
pub mod loader;
pub mod preload;
pub mod store;

pub use inspect::inspect_wasm_remote_zarr;
#[cfg(target_arch = "wasm32")]
pub(crate) use loader::CONCURRENT_FETCHES;
pub use loader::load_one_wasm_with_progress;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_STORES: std::cell::RefCell<HashMap<String, Arc<WasmZarrBlockStore>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
static WASM_STORES_DESKTOP: OnceLock<Mutex<HashMap<String, Arc<WasmZarrBlockStore>>>> =
    OnceLock::new();

/// WASM-compatible Zarr BlockStore backed by in-memory metadata and on-demand chunk fetching.
pub struct WasmZarrBlockStore {
    pub base_url: String,
    pub memory_store: ReadableWritableListableStorage,
    pub metadata: RwLock<Option<DatasetMetadata>>,
    pub is_local_notice: bool,
    pub generic_store: crate::data::backends::zarr::GenericZarrBlockStore,
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
                let memory_store: ReadableWritableListableStorage = Arc::new(MemoryStore::new());
                let generic_store = crate::data::backends::zarr::GenericZarrBlockStore::new(
                    memory_store.clone(),
                    &clean_url,
                    "zarr",
                    "Zarr",
                );
                let new_store = Arc::new(Self {
                    base_url: clean_url.clone(),
                    memory_store,
                    metadata: RwLock::new(None),
                    is_local_notice: false,
                    generic_store,
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
            let memory_store: ReadableWritableListableStorage = Arc::new(MemoryStore::new());
            let generic_store = crate::data::backends::zarr::GenericZarrBlockStore::new(
                memory_store.clone(),
                &clean_url,
                "zarr",
                "Zarr",
            );
            let new_store = Arc::new(Self {
                base_url: clean_url.clone(),
                memory_store,
                metadata: RwLock::new(None),
                is_local_notice: false,
                generic_store,
            });
            guard.insert(clean_url, new_store.clone());
            new_store
        }
    }

    pub fn open_local_notice(path: &str) -> Arc<Self> {
        let memory_store: ReadableWritableListableStorage = Arc::new(MemoryStore::new());
        let generic_store = crate::data::backends::zarr::GenericZarrBlockStore::new(
            memory_store.clone(),
            path,
            "zarr",
            "Zarr",
        );
        Arc::new(Self {
            base_url: path.to_string(),
            memory_store,
            metadata: RwLock::new(None),
            is_local_notice: true,
            generic_store,
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
}
