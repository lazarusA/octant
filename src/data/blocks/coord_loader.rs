//! Loading one variable's dimension coordinates in the background, so datasets open with
//! their metadata alone and a variable's coordinates arrive when it is chosen.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use crate::data::StoreHandle;
use crate::data::metadata::{CoordValues, VariableInfo};

/// The coordinates read for `variable` of the dataset `source_id`.
pub struct CoordResult {
    pub source_id: String,
    pub variable: String,
    pub coords: Result<HashMap<String, CoordValues>, String>,
}

/// Queues coordinate reads, at most once per `(dataset, variable)`, and hands back results.
pub struct CoordinateLoader {
    tx: SyncSender<CoordResult>,
    rx: Receiver<CoordResult>,
    requested: HashSet<(String, String)>,
    pending: usize,
}

impl Default for CoordinateLoader {
    fn default() -> Self {
        let (tx, rx) = sync_channel(64);
        Self {
            tx,
            rx,
            requested: HashSet::new(),
            pending: 0,
        }
    }
}

impl CoordinateLoader {
    /// Reads `variable`'s coordinates from `store` in the background; `false` when they
    /// were already requested for the dataset `source_id`.
    pub fn request(&mut self, source_id: &str, store: StoreHandle, variable: VariableInfo) -> bool {
        if !self
            .requested
            .insert((source_id.to_string(), variable.name.clone()))
        {
            return false;
        }
        self.pending += 1;
        let (tx, source_id) = (self.tx.clone(), source_id.to_string());

        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::TaskExecutor::spawn_background(move || {
            let coords = store
                .variable_coordinates(&variable)
                .map_err(|e| e.to_string());
            let _ = tx.send(CoordResult {
                source_id,
                variable: variable.name,
                coords,
            });
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let coords = load_wasm(&store, &variable).await;
            let _ = tx.send(CoordResult {
                source_id,
                variable: variable.name,
                coords,
            });
        });
        true
    }

    /// Results that arrived since the last poll. A failed read may be requested again.
    pub fn poll(&mut self) -> Vec<CoordResult> {
        let results: Vec<CoordResult> = self.rx.try_iter().collect();
        self.pending = self.pending.saturating_sub(results.len());
        for failed in results.iter().filter(|r| r.coords.is_err()) {
            self.requested
                .remove(&(failed.source_id.clone(), failed.variable.clone()));
        }
        results
    }

    pub fn pending_count(&self) -> usize {
        self.pending
    }

    /// Forgets what was requested for the dataset `source_id` (re-inspected or removed), so
    /// its variables are read again.
    pub fn forget(&mut self, source_id: &str) {
        self.requested.retain(|(id, _)| id != source_id);
    }
}

/// In the browser, Zarr and Icechunk coordinates are fetched into memory first; other
/// stores already hold them.
#[cfg(target_arch = "wasm32")]
async fn load_wasm(
    store: &StoreHandle,
    variable: &VariableInfo,
) -> Result<HashMap<String, CoordValues>, String> {
    use crate::data::DataSourceKind;
    use crate::data::backends::icechunk::wasm::WasmIcechunkBlockStore;
    use crate::data::backends::zarr::WasmZarrBlockStore;
    let source = store.source();
    let url = source
        .uri
        .trim_start_matches("icechunk+")
        .trim_end_matches('/');
    let icechunk = source.uri.starts_with("icechunk+");
    match source.kind {
        DataSourceKind::RemoteIcechunk | DataSourceKind::LocalIcechunk => {
            let store = WasmIcechunkBlockStore::get_or_create(url);
            Ok(store.load_variable_coordinates(variable).await)
        }
        DataSourceKind::RemoteZarr | DataSourceKind::LocalZarr if icechunk => {
            let store = WasmIcechunkBlockStore::get_or_create(url);
            Ok(store.load_variable_coordinates(variable).await)
        }
        DataSourceKind::RemoteZarr | DataSourceKind::LocalZarr => {
            let store = WasmZarrBlockStore::get_or_create(url);
            Ok(store.load_variable_coordinates(variable).await)
        }
        _ => store
            .variable_coordinates(variable)
            .map_err(|e| e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
    use crate::data::octant_block::OctantBlock;
    use crate::data::slice_request::SliceRequest;

    struct NoBlocks;

    impl BlockStore for NoBlocks {
        fn backend_name(&self) -> &str {
            "none"
        }
        fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
            Ok(Vec::new())
        }
        fn fetch_block_with_progress(
            &self,
            _request: &SliceRequest,
            _on_progress: ProgressCallback,
        ) -> Result<OctantBlock, BlockStoreError> {
            Err("no blocks".into())
        }
    }

    #[test]
    fn a_variable_is_requested_once_until_its_dataset_is_forgotten() {
        let source = crate::data::DataSource::new(
            "none",
            crate::data::DataSourceKind::Procedural,
            "none://",
            "Store",
        );
        let store = StoreHandle::new(source, Arc::new(NoBlocks));
        let var = VariableInfo {
            name: "t2m".into(),
            ..Default::default()
        };
        let mut loader = CoordinateLoader::default();
        assert!(loader.request("none", store.clone(), var.clone()));
        assert!(!loader.request("none", store.clone(), var.clone()));
        loader.forget("none");
        assert!(loader.request("none", store, var));
    }
}
