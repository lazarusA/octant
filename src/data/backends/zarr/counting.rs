//! A remote store that counts what it reads into `blocks::progress`: each
//! read (a chunk, or one inner chunk's range of a shard) and its bytes, so
//! the status bar shows a desktop fetch's progress as the browser's. Only
//! reads of the arrays a fetch planned (`plan`) count as arrived chunks, so
//! coordinate and metadata reads running beside them never do; their shard
//! index reads do, and `progress` keeps the percent below 100 until the
//! fetches end.

use zarrs::storage::byte_range::ByteRangeIterator;
use zarrs::storage::{
    Bytes, ListableStorageTraits, MaybeBytes, MaybeBytesIterator, OffsetBytesIterator,
    ReadableStorageTraits, ReadableWritableListableStorage, StorageError, StoreKey, StoreKeys,
    StoreKeysPrefixes, StorePrefix, WritableStorageTraits,
};

use crate::data::blocks::progress;

pub(super) struct CountingStore(pub(super) ReadableWritableListableStorage);

fn count(key: &StoreKey, reads: usize, bytes: usize) {
    if progress::is_planned(key.as_str()) {
        progress::arrived(reads as u64);
    }
    progress::bytes(bytes as u64);
}

impl ReadableStorageTraits for CountingStore {
    fn get(&self, key: &StoreKey) -> Result<MaybeBytes, StorageError> {
        let value = self.0.get(key)?;
        count(key, 1, value.as_ref().map_or(0, Bytes::len));
        Ok(value)
    }

    fn get_partial_many<'a>(
        &'a self,
        key: &StoreKey,
        byte_ranges: ByteRangeIterator<'a>,
    ) -> Result<MaybeBytesIterator<'a>, StorageError> {
        let Some(values) = self.0.get_partial_many(key, byte_ranges)? else {
            count(key, 1, 0);
            return Ok(None);
        };
        let values: Vec<_> = values.collect();
        let bytes = values
            .iter()
            .map(|b| b.as_ref().map_or(0, Bytes::len))
            .sum();
        count(key, values.len(), bytes);
        Ok(Some(Box::new(values.into_iter())))
    }

    fn size_key(&self, key: &StoreKey) -> Result<Option<u64>, StorageError> {
        self.0.size_key(key)
    }

    fn supports_get_partial(&self) -> bool {
        self.0.supports_get_partial()
    }
}

impl ListableStorageTraits for CountingStore {
    fn list(&self) -> Result<StoreKeys, StorageError> {
        self.0.list()
    }

    fn list_prefix(&self, prefix: &StorePrefix) -> Result<StoreKeys, StorageError> {
        self.0.list_prefix(prefix)
    }

    fn list_dir(&self, prefix: &StorePrefix) -> Result<StoreKeysPrefixes, StorageError> {
        self.0.list_dir(prefix)
    }

    fn size(&self) -> Result<u64, StorageError> {
        self.0.size()
    }

    fn size_prefix(&self, prefix: &StorePrefix) -> Result<u64, StorageError> {
        self.0.size_prefix(prefix)
    }
}

impl WritableStorageTraits for CountingStore {
    fn set(&self, key: &StoreKey, value: Bytes) -> Result<(), StorageError> {
        self.0.set(key, value)
    }

    fn set_partial_many(
        &self,
        key: &StoreKey,
        offset_values: OffsetBytesIterator,
    ) -> Result<(), StorageError> {
        self.0.set_partial_many(key, offset_values)
    }

    fn erase(&self, key: &StoreKey) -> Result<(), StorageError> {
        self.0.erase(key)
    }

    fn erase_many(&self, keys: &[StoreKey]) -> Result<(), StorageError> {
        self.0.erase_many(keys)
    }

    fn erase_prefix(&self, prefix: &StorePrefix) -> Result<(), StorageError> {
        self.0.erase_prefix(prefix)
    }

    fn supports_set_partial(&self) -> bool {
        self.0.supports_set_partial()
    }
}

/// Plans the chunks `request` reads from `array` (its inner chunks when
/// sharded), so reads under its path count as they arrive.
pub(super) fn plan(
    array: &super::storage::ZarrArrayHandle,
    request: &crate::data::slice_request::SliceRequest,
) {
    let shape = array.shape();
    let inner = zarrs::array::ArrayShardedExt::effective_subchunk_shape(array);
    let Some(chunk) = inner.or_else(|| array.chunk_shape(&vec![0; shape.len()]).ok()) else {
        return;
    };
    let chunk: Vec<u64> = chunk.iter().map(|c| c.get()).collect();
    let Ok(subset) = request.to_array_subset(shape) else {
        return;
    };
    let chunks = chunks_over(&subset.to_ranges(), &chunk);
    progress::plan_array(&key_prefix(array.path().as_str()), chunks);
}

/// The store keys under the array at `path` (`/0/t2m` -> `0/t2m/`).
fn key_prefix(path: &str) -> String {
    match path.trim_matches('/') {
        "" => String::new(),
        path => format!("{path}/"),
    }
}

/// How many chunks of shape `chunk` the window `ranges` touches.
fn chunks_over(ranges: &[std::ops::Range<u64>], chunk: &[u64]) -> u64 {
    ranges
        .iter()
        .zip(chunk)
        .map(|(r, &c)| match r.end > r.start && c > 0 {
            true => (r.end - 1) / c - r.start / c + 1,
            false => 0,
        })
        .product()
}

#[cfg(test)]
mod tests {
    use super::{chunks_over, key_prefix};

    #[test]
    fn an_array_owns_the_keys_under_its_path() {
        assert_eq!(key_prefix("/0/t2m"), "0/t2m/");
        assert_eq!(key_prefix("/"), "", "the root array: every key");
    }

    #[test]
    fn a_window_counts_the_chunks_it_touches() {
        assert_eq!(chunks_over(&[0..64, 0..64], &[64, 64]), 1);
        assert_eq!(
            chunks_over(&[63..65, 10..11], &[64, 64]),
            2,
            "across an edge"
        );
        assert_eq!(
            chunks_over(&[13..14, 0..5529, 0..6360], &[64, 64, 64]),
            87 * 100
        );
        assert_eq!(chunks_over(&[5..5, 0..64], &[64, 64]), 0, "empty");
    }
}
