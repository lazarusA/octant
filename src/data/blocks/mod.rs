//! Block caching, streaming, prefetching, and storage abstraction.

pub mod cache;
pub mod coord_loader;
pub mod key;
pub mod loader;
pub mod prefetch;
pub mod request;
pub mod store;
pub mod summary;
#[cfg(test)]
mod tests;

pub use cache::BlockCache;
pub use coord_loader::{CoordResult, CoordinateLoader};
pub use key::BlockCacheKey;
pub use loader::{BlockBatchOutcome, BlockLoadOutcome, BlockLoader};
pub use prefetch::{BlockPrefetcher, PrefetchResult};
pub use request::{BlockRequest, BlockRequestBatch, BlockResult};
pub use store::{BlockStore, BlockStoreError, ProgressCallback};
pub use summary::VariableCacheSummary;
