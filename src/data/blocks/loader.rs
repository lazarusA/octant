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

    /// The browser's [`load_one`](Self::load_one), reporting downloaded bytes
    /// (`WebFormat::load_block`).
    #[cfg(target_arch = "wasm32")]
    pub async fn load_one_wasm(
        request: &BlockRequest,
        on_progress: super::store::ProgressCallback<'_>,
    ) -> Result<OctantBlock, BlockStoreError> {
        let format = crate::data::backends::web::WebFormat::of(request.store.source());
        format.load_block(request, on_progress).await
    }
}
