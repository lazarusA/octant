//! Re-exports and type aliases bridging to `crate::data::block_cache`.

pub use crate::data::block_cache::{BlockCache, BlockCacheKey};
pub use crate::data::block_prefetch::{BlockPrefetcher, PrefetchResult};

/// Type alias for backward compatibility.
pub type BlockLruCache = BlockCache;
