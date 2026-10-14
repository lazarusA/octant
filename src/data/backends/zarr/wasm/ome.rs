//! OME-Zarr discovery in the browser: walks run over the metadata fetched so
//! far, and the keys each walk still needs are fetched before the next.

use futures::StreamExt;

use super::{CONCURRENT_METADATA_FETCHES, WasmZarrBlockStore};
use crate::data::backends::http::fetch_url_bytes;
use crate::data::ome::{Discovery, MetaReader, Outcome, Prefetch, discover};

/// Walks before giving up (each nesting level of a store takes about one).
/// (A container of 4096 series takes 64 passes to probe.)
const MAX_PASSES: usize = 96;

impl WasmZarrBlockStore {
    /// The OME-Zarr images, labels, wells and series of the store; `None`
    /// when its root is not an OME-Zarr group.
    pub async fn discover_ome(&self) -> Option<Discovery> {
        let prefetch = Prefetch::default();
        // One reader for every walk: keys read in one are not read again.
        let reader = MetaReader::prefetched(self.memory_store.clone(), &prefetch);
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
                .map(|key| async move {
                    let url = format!("{base}/{key}");
                    let res = match fetch_url_bytes(&url).await {
                        // A failed request (not a missing key) is tried again once.
                        Err(e) if !e.contains("HTTP 404") => fetch_url_bytes(&url).await,
                        res => res,
                    };
                    (key, res)
                })
                .buffer_unordered(CONCURRENT_METADATA_FETCHES);
            while let Some((key, res)) = fetches.next().await {
                match res {
                    Ok(bytes) if self.insert_key_bytes(&key, &bytes).is_ok() => {}
                    Ok(_) => {
                        prefetch.mark_absent(key);
                    }
                    Err(e) => {
                        if !e.contains("HTTP 404") {
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
