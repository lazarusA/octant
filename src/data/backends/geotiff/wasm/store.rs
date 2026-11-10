//! The browser's GeoTIFF stores, one per URL, each opened once.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::channel::oneshot;

use crate::data::DatasetMetadata;
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

use super::super::reader::WasmHttpTiffReader;
use super::super::store::GeoTiffBlockStore;

// Per thread: the browser has one; the desktop uses these stores only in tests.
thread_local! {
    static WASM_GEOTIFF_STORES: std::cell::RefCell<HashMap<String, Arc<WasmGeoTiffBlockStore>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// What a store's file is: unread, being opened (with the opens waiting for it), or open.
enum State {
    Closed,
    Opening(Vec<oneshot::Sender<Opened>>),
    Open(Arc<GeoTiffBlockStore>),
}

type Opened = Result<Arc<GeoTiffBlockStore>, String>;

/// The browser's GeoTIFF/COG store: its headers read by HTTP ranges once, by the first
/// inspection or block load, every other caller waiting for that read (`open`).
pub struct WasmGeoTiffBlockStore {
    pub url: String,
    state: Mutex<State>,
}

impl WasmGeoTiffBlockStore {
    fn new(url: &str) -> Arc<Self> {
        Arc::new(Self {
            url: url.to_string(),
            state: Mutex::new(State::Closed),
        })
    }

    /// The opened file, reading its headers unless another caller is reading them, then
    /// waiting for that read. A failed read is not kept: the next caller reads again.
    pub async fn open(&self) -> Opened {
        let joined = {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            match &mut *state {
                State::Open(store) => return Ok(store.clone()),
                State::Opening(waiting) => {
                    let (tx, rx) = oneshot::channel();
                    waiting.push(tx);
                    Some(rx)
                }
                State::Closed => {
                    *state = State::Opening(Vec::new());
                    None
                }
            }
        };
        if let Some(rx) = joined {
            return rx
                .await
                .unwrap_or_else(|_| Err(format!("'{}' did not open", self.url)));
        }
        let reader = Arc::new(WasmHttpTiffReader::new(&self.url));
        let opened = GeoTiffBlockStore::from_reader(&self.url, reader)
            .await
            .map(Arc::new)
            .map_err(|e| format!("Failed to load GeoTIFF at '{}': {e}", self.url));
        let next = match &opened {
            Ok(store) => State::Open(store.clone()),
            Err(_) => State::Closed,
        };
        let previous = {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            std::mem::replace(&mut *state, next)
        };
        if let State::Opening(waiting) = previous {
            for tx in waiting {
                let _ = tx.send(opened.clone());
            }
        }
        opened
    }

    /// The store once open.
    fn opened(&self) -> Result<Arc<GeoTiffBlockStore>, BlockStoreError> {
        match &*self.state.lock().unwrap_or_else(|p| p.into_inner()) {
            State::Open(store) => Ok(store.clone()),
            _ => Err(format!("Metadata for '{}' is loading...", self.url).into()),
        }
    }

    pub fn get_or_create(url: &str) -> Arc<Self> {
        let clean_url = url.trim().to_string();
        WASM_GEOTIFF_STORES.with(|stores| {
            let mut map = stores.borrow_mut();
            if let Some(store) = map.get(&clean_url) {
                store.clone()
            } else {
                let new_store = Self::new(&clean_url);
                map.insert(clean_url, new_store.clone());
                new_store
            }
        })
    }

    /// Drops the store of `url` from the registry (its dataset closed).
    pub fn release(url: &str) {
        let clean_url = url.trim();
        let _released = WASM_GEOTIFF_STORES.with(|stores| stores.borrow_mut().remove(clean_url));
    }
}

impl BlockStore for WasmGeoTiffBlockStore {
    fn backend_name(&self) -> &str {
        "GeoTIFF (Web COG)"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.opened()?.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.opened()?.inspect()
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.opened()?.fetch_block(request)
    }
}
