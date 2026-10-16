//! Fetching the parts of shards a window needs into the browser's store:
//! each shard's index (one suffix or prefix range), then the byte ranges of
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
use crate::data::backends::http::{fetch_url_byte_range, fetch_url_tail, is_not_found};
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
    /// `shards` not yet held. A shard the server lacks (404) is empty.
    pub(crate) async fn fetch_shard_parts(
        &self,
        spec: &ShardIndexSpec,
        shards: &[ShardRef],
        window: &[Range<u64>],
        on_progress: &mut ProgressCallback<'_>,
    ) -> Result<(), BlockStoreError> {
        let mut wanted: Vec<(&ShardRef, StoreKey, u64, Range<u64>)> = Vec::new();
        for shard in shards {
            let key = StoreKey::new(&shard.key).map_err(|e| format!("{e}"))?;
            let Some((size, index)) = self.shard_index(spec, &key, shard).await? else {
                continue;
            };
            let Some(entries) = parse_index(spec, &index) else {
                return Err(format!("Unreadable shard index of '{}'", shard.url).into());
            };
            let ranges = window_ranges(spec, &shard.indices, window, &entries)
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

    /// The size and index of `shard` (store key `key`), held or fetched;
    /// `None` for a shard the server lacks.
    async fn shard_index(
        &self,
        spec: &ShardIndexSpec,
        key: &StoreKey,
        shard: &ShardRef,
    ) -> Result<Option<(u64, Vec<u8>)>, BlockStoreError> {
        let len = spec.index_len();
        if let Ok(Some(size)) = self.partial_store.size_key(key) {
            let range = spec.index_range(size);
            if let Ok(Some(index)) = self
                .partial_store
                .get_partial(key, ByteRange::FromStart(range.start, Some(len)))
            {
                return Ok(Some((size, index.to_vec())));
            }
        }
        let url = shard.url.as_str();
        let fetched = match (&shard.span, spec.at_end) {
            (Some(span), _) => {
                let size = span.end - span.start;
                let range = spec.index_range(size);
                let from = span.start + range.start;
                let index = fetch_url_byte_range(url, from, range.end - range.start).await;
                index.map(|b| (b, size))
            }
            (None, true) => fetch_url_tail(url, len).await,
            // A prefix index: the size comes from the tail's Content-Range.
            (None, false) => match fetch_url_tail(url, 1).await {
                Ok((_, size)) => fetch_url_byte_range(url, 0, len).await.map(|b| (b, size)),
                Err(e) => Err(e),
            },
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
