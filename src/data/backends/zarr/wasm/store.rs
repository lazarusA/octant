//! BlockStore trait implementation for `WasmZarrBlockStore`.

use super::WasmZarrBlockStore;
use crate::data::DatasetMetadata;
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

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

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.generic_store.fetch_block(request)
    }
}
