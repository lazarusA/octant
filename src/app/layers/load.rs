//! A layer's pending block request.

use super::VariableSelection;
use crate::data::{BlockCacheKey, SliceRequest};

#[derive(Debug, Clone, Default)]
pub struct LoadState {
    /// Key of the block requested for the view, until it arrives.
    pub block_key: Option<BlockCacheKey>,
    /// The latest requested view: blocks of an older selection, variable or
    /// plot layout are not projected into it.
    pub slice_request: Option<SliceRequest>,
    /// The base layer's staged selection its pending block was requested
    /// from: the plot takes it on arrival, whatever is staged by then.
    pub requested: Option<VariableSelection>,
    /// Step the requested block is shown at when it arrives (playback may
    /// move on meanwhile).
    pub pending_target_step: Option<usize>,
    /// The layer's last fetch failed: an overlay no longer holds playback
    /// nor prefetches ahead until one of its blocks arrives (it retries
    /// once per step).
    pub failed: bool,
}
