//! Loading one block through its own store.
//!
//! `BlockLoader` is cache-agnostic: callers check `BlockCache` before
//! calling here, so loading has no cache side effects of its own.

use super::request::BlockRequest;
use super::store::BlockStoreError;
use crate::data::octant_block::OctantBlock;

pub struct BlockLoader;

impl BlockLoader {
    /// Loads a single block through its own store (desktop stores report
    /// progress to `blocks::progress` themselves).
    pub fn load_one(request: &BlockRequest) -> Result<OctantBlock, BlockStoreError> {
        request.store.fetch(&request.slice)
    }

    /// The browser's [`load_one`](Self::load_one), reporting downloaded bytes:
    /// each format's async loader fetches the block's chunks, then decodes them.
    #[cfg(target_arch = "wasm32")]
    pub async fn load_one_wasm(
        request: &BlockRequest,
        on_progress: super::store::ProgressCallback<'_>,
    ) -> Result<OctantBlock, BlockStoreError> {
        use crate::data::DataSourceKind as Kind;
        use crate::data::backends::{geotiff, icechunk, zarr};
        match request.store.source().kind {
            Kind::RemoteIcechunk | Kind::LocalIcechunk => {
                icechunk::wasm::load_one_icechunk_wasm_with_progress(request, on_progress).await
            }
            Kind::RemoteGeoTiff | Kind::LocalGeoTiff => {
                geotiff::wasm::load_one_geotiff_wasm_with_progress(request, on_progress).await
            }
            Kind::Procedural => request
                .store
                .fetch_with_progress(&request.slice, on_progress),
            _ => zarr::load_one_wasm_with_progress(request, on_progress).await,
        }
    }
}
