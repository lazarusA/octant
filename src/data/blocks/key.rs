//! Block cache key and selection matching.

use crate::data::slice_request::{DimensionSelection, SliceRequest};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockCacheKey {
    /// Identity of the opened source this block came from (`DataSource::id`,
    /// reached via `StoreHandle::source().id`). Two `StoreHandle`s opened
    /// against the same source resolve to the same key.
    pub source_id: String,
    pub variable_name: String,
    pub selections: Vec<DimensionSelection>,
}

impl BlockCacheKey {
    pub fn new(source_id: impl Into<String>, slice: &SliceRequest) -> Self {
        Self {
            source_id: source_id.into(),
            variable_name: slice.variable.clone(),
            selections: slice.selections.clone(),
        }
    }
}

pub(crate) fn selections_match_except_anim(
    cached: &[DimensionSelection],
    requested: &[DimensionSelection],
    anim_dim: Option<usize>,
) -> bool {
    if cached.len() != requested.len() {
        return false;
    }
    for (i, (c, r)) in cached.iter().zip(requested.iter()).enumerate() {
        if anim_dim == Some(i) {
            continue;
        }
        if c != r {
            return false;
        }
    }
    true
}
