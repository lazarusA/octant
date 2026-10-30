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
    cache: &super::MetaCache,
) -> Option<super::Discovery> {
    use super::discover::{Outcome, discover};
    let found = walk_concurrent(storage, cache, MAX_PASSES, |reader| {
        match discover(reader, "") {
            Outcome::Found(found) => Some(Some(found)),
            Outcome::NotOme => Some(None),
            Outcome::Pending => None,
        }
    });
    if found.is_none() {
        log::warn!("OME-Zarr discovery did not settle");
    }
    found.flatten()
}

/// Runs `walk` over the keys of `storage` read so far until it settles
/// (`Some`), reading the keys each walk still needs concurrently before the
/// next; `None` when it has not settled after `passes` walks or needs no
/// more keys. Documents already in the store's `cache` are not read; a key
/// is cached missing only once read and not found.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn walk_concurrent<T>(
    storage: std::sync::Arc<dyn zarrs::storage::ReadableStorageTraits>,
    cache: &super::MetaCache,
    passes: usize,
    mut walk: impl FnMut(&super::reader::MetaReader<'_>) -> Option<T>,
) -> Option<T> {
    use super::reader::{MetaReader, Prefetch};
    use rayon::prelude::*;
    use std::sync::Arc;
    use zarrs::storage::store::MemoryStore;
    use zarrs::storage::{StoreKey, WritableStorageTraits};
    let memory = Arc::new(MemoryStore::new());
    let prefetch = Prefetch::default();
    // One reader for every walk: keys read in one are not read again.
    let reader = MetaReader::prefetched(memory.clone(), &prefetch).with_cache(cache);
    for _ in 0..passes {
        let settled = walk(&reader);
        let wanted = prefetch.take_pending();
        if settled.is_some() || wanted.is_empty() {
            return settled;
        }
        let read: Vec<_> = wanted
            .into_par_iter()
            .map(|key| {
                let store_key = StoreKey::new(key.as_str()).ok();
                let read = store_key.as_ref().map(|k| storage.get(k));
                (key, store_key, read)
            })
            .collect();
        for (key, store_key, read) in read {
            let stored = match (store_key, read) {
                (Some(k), Some(Ok(Some(bytes)))) => memory.set(&k, bytes).is_ok(),
                // A read that failed is not a missing key: the store's cache
                // must not remember it.
                (_, Some(Err(_))) => {
                    prefetch.mark_failed(key);
                    continue;
                }
                _ => false,
            };
            if !stored {
                prefetch.mark_absent(key);
            }
        }
    }
    None
}
