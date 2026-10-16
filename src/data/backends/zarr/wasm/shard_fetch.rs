//! Fetching the parts of shards a window needs into the browser's store:
//! each shard's index (one suffix or prefix range), then the byte ranges of
//! the inner chunks the window touches, never the whole shard.

use std::ops::Range;

use futures::StreamExt;
use zarrs::storage::{Bytes, StoreKey};

use super::WasmZarrBlockStore;
use super::loader::CONCURRENT_FETCHES;
use super::shard_plan::{ShardIndexSpec, merge, parse_index};
use crate::data::backends::http::{fetch_url_byte_range, fetch_url_tail, is_not_found};
use crate::data::blocks::{BlockStoreError, ProgressCallback};

/// One shard a window touches: its store key, URL and grid indices.
pub(super) struct ShardRef {
    pub key: String,
    pub url: String,
    pub indices: Vec<u64>,
}

impl WasmZarrBlockStore {
    /// Fetches the index and the window's inner chunks of every shard in
    /// `shards` not yet held. A shard the server lacks (404) is empty.
    pub(super) async fn fetch_shard_parts(
        &self,
        spec: &ShardIndexSpec,
        shards: &[ShardRef],
        window: &[Range<u64>],
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let mut wanted: Vec<(StoreKey, String, u64, Range<u64>)> = Vec::new();
        for shard in shards {
            let key = StoreKey::new(&shard.key).map_err(|e| format!("{e}"))?;
            let Some((size, index)) = self.shard_index(spec, &key, &shard.url).await? else {
                continue;
            };
            let Some(entries) = parse_index(spec, &index) else {
                return Err(format!("Unreadable shard index of '{}'", shard.url).into());
            };
            let ranges = spec
                .inner_chunks(&shard.indices, window)
                .into_iter()
                .filter_map(|i| entries.get(i as usize).copied().flatten())
                .map(|(offset, len)| offset..offset + len)
                .filter(|r| !self.partial_store.holds(&key, r.start, r.end))
                .collect();
            for r in merge(ranges) {
                wanted.push((key.clone(), shard.url.clone(), size, r));
            }
        }
        let mut fetches = futures::stream::iter(&wanted)
            .map(|(key, url, size, r)| async move {
                let bytes = fetch_url_byte_range(url, r.start, r.end - r.start).await;
                (key, url, *size, r.start, bytes)
            })
            .buffer_unordered(CONCURRENT_FETCHES);
        while let Some((key, url, size, offset, res)) = fetches.next().await {
            let bytes = res.map_err(|e| format!("Failed to fetch chunks of '{url}': {e}"))?;
            if let Some(cb) = on_progress.as_mut() {
                cb(bytes.len() as u64);
            }
            self.partial_store
                .insert_part(key, size, offset, Bytes::from(bytes));
        }
        Ok(())
    }

    /// The size and index of shard `key`, held or fetched; `None` for a
    /// shard the server lacks.
    async fn shard_index(
        &self,
        spec: &ShardIndexSpec,
        key: &StoreKey,
        url: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, BlockStoreError> {
        let len = spec.index_len();
        if let Some(size) = self.partial_store.part_size(key) {
            let range = spec.index_range(size);
            if let Ok(Some(index)) = zarrs::storage::ReadableStorageTraits::get_partial(
                self.partial_store.as_ref(),
                key,
                zarrs::storage::byte_range::ByteRange::FromStart(range.start, Some(len)),
            ) {
                return Ok(Some((size, index.to_vec())));
            }
        }
        let fetched = if spec.at_end {
            fetch_url_tail(url, len).await
        } else {
            // A prefix index: the size comes from the tail's Content-Range.
            match fetch_url_tail(url, 1).await {
                Ok((_, size)) => fetch_url_byte_range(url, 0, len).await.map(|b| (b, size)),
                Err(e) => Err(e),
            }
        };
        let (index, size) = match fetched {
            Ok(found) => found,
            Err(e) if is_not_found(&e) => return Ok(None),
            Err(e) => return Err(format!("Failed to fetch the index of '{url}': {e}").into()),
        };
        let start = spec.index_range(size).start;
        self.partial_store
            .insert_part(key, size, start, Bytes::from(index.clone()));
        Ok(Some((size, index)))
    }
}
