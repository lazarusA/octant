//! Cancelled fetches (`BlockPrefetcher::cancel`): a running one still ends
//! into the channel, flagged unrequested, and never clears a newer request
//! for the same block.

use super::prefetch_tests::{Store, request, until};
use crate::data::blocks::{BlockCache, BlockPrefetcher};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, mpsc};

#[test]
fn a_cancelled_running_fetch_is_no_longer_pending_but_still_delivers() {
    let (release, hold) = mpsc::channel();
    let store = Store {
        fetches: Arc::default(),
        fail: false,
        hold: Some(Mutex::new(hold)),
    };
    let (req, fetches) = request(store);
    let cache = BlockCache::new(1024 * 1024);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(2);

    assert!(prefetcher.request(req(), &cache));
    until(|| fetches.load(Ordering::SeqCst) == 1);
    assert!(prefetcher.cancel(&req().cache_key()));
    assert!(!prefetcher.is_pending(&req().cache_key()));
    // Already downloading: it ends into the channel, for the cache.
    let _ = release.send(());
    let mut results = Vec::new();
    until(|| {
        results.extend(prefetcher.poll());
        !results.is_empty()
    });
    assert!(results[0].result.is_ok());
    assert!(results[0].unrequested, "nobody waits on it");
    assert!(
        !prefetcher.cancel(&req().cache_key()),
        "nothing left to cancel"
    );
}

#[test]
fn a_cancelled_fetch_ending_late_leaves_a_newer_request_pending() {
    let (release, hold) = mpsc::channel();
    let store = Store {
        fetches: Arc::default(),
        fail: false,
        hold: Some(Mutex::new(hold)),
    };
    let (req, fetches) = request(store);
    let cache = BlockCache::new(1024 * 1024);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(4);
    let key = req().cache_key();

    assert!(prefetcher.request(req(), &cache));
    until(|| fetches.load(Ordering::SeqCst) == 1);
    prefetcher.cancel(&key);
    // Asked again while the cancelled one still downloads.
    assert!(prefetcher.request(req(), &cache));
    let _ = release.send(());
    let mut first = Vec::new();
    until(|| {
        first.extend(prefetcher.poll());
        !first.is_empty()
    });
    assert!(first[0].unrequested, "the cancelled read ends first");
    assert!(
        prefetcher.is_pending(&key),
        "the newer request is still tracked"
    );
    let _ = release.send(());
    let mut second = Vec::new();
    until(|| {
        second.extend(prefetcher.poll());
        !second.is_empty()
    });
    assert!(!second[0].unrequested);
    assert!(!prefetcher.is_pending(&key));
}
