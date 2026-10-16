//! How an array's subsets are read: straight from the store, through a
//! decoded-chunk cache, or for a sharded array through its shard indexes.

use crate::data::blocks::BlockStoreError;
use zarrs::array::chunk_cache::{ChunkCache, ChunkCacheDecodedLruSizeLimit};
use zarrs::array::{Array, ArraySubset, CodecOptions, FromArrayBytes};
use zarrs::array::{ArrayShardedReadableExt, ArrayShardedReadableExtCache};
use zarrs::storage::ReadableStorageTraits;

pub(super) trait SubsetRetriever {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError>;
}

impl<TStorage: ?Sized + ReadableStorageTraits + 'static> SubsetRetriever for Array<TStorage> {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.retrieve_array_subset(subset)
            .map_err(|e| format!("{e}").into())
    }
}

impl SubsetRetriever for ChunkCacheDecodedLruSizeLimit {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.retrieve_array_subset(subset, &CodecOptions::default())
            .map_err(|e| format!("{e}").into())
    }
}

/// How an array's subsets are read.
#[derive(Clone, Copy)]
pub enum ChunkReader<'a> {
    /// Straight from the store, whole chunks at a time.
    Direct,
    /// Through a cache of decoded chunks.
    Cached(&'a ChunkCacheDecodedLruSizeLimit),
    /// A sharded array: only the inner chunks a subset touches, located
    /// through the shard indexes the cache keeps.
    Sharded(&'a ArrayShardedReadableExtCache),
}

/// A sharded array read through its shard indexes.
pub(super) struct Sharded<'a, TStorage: ?Sized> {
    pub array: &'a Array<TStorage>,
    pub cache: &'a ArrayShardedReadableExtCache,
}

impl<TStorage: ?Sized + ReadableStorageTraits + 'static> SubsetRetriever for Sharded<'_, TStorage> {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.array
            .retrieve_array_subset_sharded_opt(self.cache, subset, &CodecOptions::default())
            .map_err(|e| format!("{e}").into())
    }
}
