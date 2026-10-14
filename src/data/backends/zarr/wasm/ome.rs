//! OME-Zarr discovery in the browser: walks run over the metadata fetched so
//! far, and the keys each walk still needs are fetched before the next.

use futures::StreamExt;

use super::{CONCURRENT_METADATA_FETCHES, WasmZarrBlockStore};
use crate::data::backends::http::{fetch_url_bytes, is_missing};
use crate::data::ome::direct::MAX_PASSES;
use crate::data::ome::{Discovery, MetaReader, Outcome, Prefetch, discover};

impl WasmZarrBlockStore {
    /// The OME-Zarr images, labels, wells and series of the store; `None`
    /// when its root is not an OME-Zarr group. With the store's Zarr `format`
    /// known, only that format's keys are asked.
    pub async fn discover_ome(&self, format: Option<u8>) -> Option<Discovery> {
        let prefetch = Prefetch::default();
        // One reader for every walk: keys read in one are not read again.
        let reader = MetaReader::prefetched(self.memory_store.clone(), &prefetch);
        if let Some(format) = format {
            reader.assume_format(format);
        }
        for _ in 0..MAX_PASSES {
            let outcome = discover(&reader, "");
            let wanted = prefetch.take_pending();
            match outcome {
                Outcome::Found(found) => return Some(found),
                Outcome::NotOme => return None,
                Outcome::Pending if wanted.is_empty() => return None,
                Outcome::Pending => {}
            }
            let base = self.base_url.trim_end_matches('/');
            let mut fetches = futures::stream::iter(wanted)
                .map(|key| fetch_key(base, key))
                .buffer_unordered(CONCURRENT_METADATA_FETCHES);
            while let Some((key, res)) = fetches.next().await {
                match res {
                    Ok(bytes) if self.insert_key_bytes(&key, &bytes).is_ok() => {}
                    res => {
                        if let Err(e) = res
                            && !is_missing(&e)
                        {
                            log::warn!("[WASM Zarr] OME-Zarr metadata '{key}' unreadable: {e}");
                        }
                        prefetch.mark_absent(key);
                    }
                }
            }
        }
        log::warn!(
            "[WASM Zarr] OME-Zarr discovery of {} did not settle",
            self.base_url
        );
        None
    }
}

/// The bytes of `key` under `base`; a failed request (not a missing key) is
/// tried again once.
async fn fetch_key(base: &str, key: String) -> (String, Result<Vec<u8>, String>) {
    let url = format!("{base}/{key}");
    let res = match fetch_url_bytes(&url).await {
        Err(e) if !is_missing(&e) => fetch_url_bytes(&url).await,
        res => res,
    };
    (key, res)
}
