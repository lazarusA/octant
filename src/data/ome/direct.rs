//! Discovery over a store that answers keys directly (desktop): walks run
//! over the keys read so far, as on the web, and the keys each walk still
//! needs are read from the store concurrently before the next.

/// Walks before giving up (each nesting level of a store takes about one;
/// a container of 4096 series takes 64 passes to probe).
pub const MAX_PASSES: usize = 96;

/// The OME-Zarr images, labels, wells and series of `storage`; `None` when
/// its root is not an OME-Zarr group.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn discover_concurrent(
    storage: std::sync::Arc<dyn zarrs::storage::ReadableStorageTraits>,
) -> Option<super::Discovery> {
    use super::discover::{Outcome, discover};
    use super::reader::{MetaReader, Prefetch};
    use rayon::prelude::*;
    use std::sync::Arc;
    use zarrs::storage::store::MemoryStore;
    use zarrs::storage::{StoreKey, WritableStorageTraits};
    let memory = Arc::new(MemoryStore::new());
    let prefetch = Prefetch::default();
    // One reader for every walk: keys read in one are not read again.
    let reader = MetaReader::prefetched(memory.clone(), &prefetch);
    for _ in 0..MAX_PASSES {
        let outcome = discover(&reader, "");
        let wanted = prefetch.take_pending();
        match outcome {
            Outcome::Found(found) => return Some(found),
            Outcome::NotOme => return None,
            Outcome::Pending if wanted.is_empty() => return None,
            Outcome::Pending => {}
        }
        let read: Vec<_> = wanted
            .into_par_iter()
            .map(|key| {
                let store_key = StoreKey::new(key.as_str()).ok();
                let bytes = store_key
                    .as_ref()
                    .and_then(|k| storage.get(k).ok().flatten());
                (key, store_key, bytes)
            })
            .collect();
        for (key, store_key, bytes) in read {
            let stored = match (store_key, bytes) {
                (Some(k), Some(bytes)) => memory.set(&k, bytes).is_ok(),
                _ => false,
            };
            if !stored {
                prefetch.mark_absent(key);
            }
        }
    }
    log::warn!("OME-Zarr discovery did not settle");
    None
}
