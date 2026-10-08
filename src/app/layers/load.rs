//! A layer's pending block request.

use crate::data::{BlockCacheKey, SliceRequest};

#[derive(Debug, Clone, Default)]
pub struct LoadState {
    /// Key of the block requested for the view, until it arrives.
    pub block_key: Option<BlockCacheKey>,
    /// The latest requested view: blocks of an older selection, variable or
    /// plot layout are not projected into it.
    pub slice_request: Option<SliceRequest>,
    /// Step the requested block is shown at when it arrives (playback may
    /// move on meanwhile).
    pub pending_target_step: Option<usize>,
}
