//! Fetching the parts of shards a window needs into the browser's store:
//! each shard's index (its size, then one range), then the byte ranges of
//! the inner chunks the window touches, never the whole shard. A shard is
//! a whole object (Zarr) or a span of one (Icechunk packs chunks into
//! larger objects and its manifests give each one's offset and length).

use std::ops::Range;

use futures::StreamExt;
use zarrs::storage::byte_range::ByteRange;
use zarrs::storage::{Bytes, ReadableStorageTraits, StoreKey};

use super::WasmZarrBlockStore;
use super::loader::CONCURRENT_FETCHES;
use super::shard_plan::{ShardIndexSpec, merge, parse_index, window_ranges};
use crate::data::backends::http::{fetch_url_byte_range, fetch_url_size, is_not_found};
use crate::data::blocks::{BlockStoreError, ProgressCallback};

/// One shard a window touches: its store key, URL and grid indices.
pub(crate) struct ShardRef {
    pub key: String,
    pub url: String,
    pub indices: Vec<u64>,
    /// The shard's bytes within the object at `url`; `None` when the shard
    /// is the whole object.
    pub span: Option<Range<u64>>,
}

impl ShardRef {
    /// Where the shard starts in the object at `url`.
    fn base(&self) -> u64 {
        self.span.as_ref().map_or(0, |s| s.start)
    }
}

impl WasmZarrBlockStore {
    /// Fetches the index and the window's inner chunks of every shard in
    /// `shards` not yet held, the indexes concurrently, then the chunks. A
    /// shard the server lacks (404) is empty.
    pub(crate) async fn fetch_shard_parts(
        &self,
        spec: &ShardIndexSpec,
        shards: &[ShardRef],
        window: &[Range<u64>],
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let indexes: Vec<_> = futures::stream::iter(shards)
            .map(|shard| async move { (shard, self.shard_index(spec, shard).await) })
            .buffer_unordered(CONCURRENT_FETCHES)
            .collect()
            .await;
        let mut wanted: Vec<(&ShardRef, StoreKey, u64, Range<u64>)> = Vec::new();
        for (shard, index) in indexes {
            let Some((key, size, index)) = index? else {
                continue;
            };
            let Some(entries) = parse_index(spec, &index) else {
                return Err(format!("Unreadable shard index of '{}'", shard.url).into());
            };
            let ranges = window_ranges(spec, &shard.indices, window, &entries, size)
                .into_iter()
                .filter(|r| !self.partial_store.holds(&key, r.start, r.end))
                .collect();
            for r in merge(ranges) {
                wanted.push((shard, key.clone(), size, r));
            }
        }
        let mut fetches = futures::stream::iter(&wanted)
            .map(|(shard, key, size, r)| async move {
                let from = shard.base() + r.start;
                let bytes = fetch_url_byte_range(&shard.url, from, r.end - r.start).await;
                (key, &shard.url, *size, r.start, bytes)
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

    /// The store key, size and index of `shard`, held or fetched (its size
    /// from its span, else a `HEAD` request; then the index by a plain
    /// range, which needs no CORS preflight); `None` for a shard the server
    /// lacks.
    async fn shard_index(
        &self,
        spec: &ShardIndexSpec,
        shard: &ShardRef,
    ) -> Result<Option<(StoreKey, u64, Bytes)>, BlockStoreError> {
        let key = StoreKey::new(&shard.key).map_err(|e| format!("{e}"))?;
        if let Ok(Some(size)) = self.partial_store.size_key(&key) {
            let range = spec.index_range(size);
            let held = ByteRange::FromStart(range.start, Some(range.end - range.start));
            if let Ok(Some(index)) = self.partial_store.get_partial(&key, held) {
                return Ok(Some((key, size, index)));
            }
        }
        let url = shard.url.as_str();
        let size = match &shard.span {
            Some(span) => Ok(span.end - span.start),
            None => fetch_url_size(url).await,
        };
        let fetched = match size {
            Ok(size) => {
                let range = spec.index_range(size);
                let from = shard.base() + range.start;
                let index = fetch_url_byte_range(url, from, range.end - range.start).await;
                index.map(|b| (size, range.start, Bytes::from(b)))
            }
            Err(e) => Err(e),
        };
        let (size, start, index) = match fetched {
            Ok(found) => found,
            Err(e) if is_not_found(&e) => return Ok(None),
            Err(e) => return Err(format!("Failed to fetch the index of '{url}': {e}").into()),
        };
        self.partial_store
            .insert_part(&key, size, start, index.clone());
        Ok(Some((key, size, index)))
    }
}
