//! The prefetcher's worker slots and failed blocks, over a store that
//! counts its fetches and can fail them or wait for a release.

use super::tests::test_block;
use crate::data::blocks::BlockCache;
use crate::data::blocks::{BlockPrefetcher, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::data::{BlockRequest, DataSource, DataSourceKind, StoreHandle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

/// Counts fetches; fails them, or waits for a release first.
struct Store {
    fetches: Arc<AtomicUsize>,
    fail: bool,
    hold: Option<Mutex<mpsc::Receiver<()>>>,
}

impl BlockStore for Store {
    fn backend_name(&self) -> &str {
        "test"
    }
    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Ok(vec!["temperature".into()])
    }
    fn fetch_block_with_progress(
        &self,
        _request: &SliceRequest,
        _on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        if let Some(rx) = self.hold.as_ref().and_then(|rx| rx.lock().ok()) {
            let _ = rx.recv();
        }
        if self.fail {
            return Err("403".into());
        }
        Ok(test_block(0))
    }
}

fn request(store: Store) -> (impl Fn() -> BlockRequest, Arc<AtomicUsize>) {
    let fetches = store.fetches.clone();
    let source = DataSource::new("test", DataSourceKind::Procedural, "test", "test");
    let handle = StoreHandle::new(source, Arc::new(store));
    let slice = SliceRequest::new("temperature", vec![DimensionSelection::range(0, 10)]);
    (
        move || BlockRequest::new(handle.clone(), slice.clone()),
        fetches,
    )
}

fn until(mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(10), "timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_failed_block_is_not_prefetched_again() {
    let store = Store {
        fetches: Arc::default(),
        fail: true,
        hold: None,
    };
    let (req, fetches) = request(store);
    let cache = BlockCache::new(1024 * 1024);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(4);

    assert!(prefetcher.request(req(), &cache));
    until(|| !prefetcher.poll().is_empty());
    // The lookahead skips it until Play, Plot or Abort clears it.
    assert!(prefetcher.failed_recently(&req().cache_key()));
    prefetcher.clear_failed();
    assert!(!prefetcher.failed_recently(&req().cache_key()));
    assert!(prefetcher.request(req(), &cache));
    until(|| !prefetcher.poll().is_empty());
    assert!(prefetcher.failed_recently(&req().cache_key()));
    assert_eq!(fetches.load(Ordering::SeqCst), 2);
}

#[test]
fn a_failed_block_is_asked_again_after_a_while() {
    let store = Store {
        fetches: Arc::default(),
        fail: true,
        hold: None,
    };
    let (req, fetches) = request(store);
    let cache = BlockCache::new(1024 * 1024);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(4);
    prefetcher.retry_after = Duration::from_millis(50);

    assert!(prefetcher.request(req(), &cache));
    until(|| !prefetcher.poll().is_empty());
    assert!(prefetcher.failed_recently(&req().cache_key()));
    // Once the wait is over (the network may be back), it is asked again.
    std::thread::sleep(Duration::from_millis(60));
    assert!(!prefetcher.failed_recently(&req().cache_key()));
    assert_eq!(fetches.load(Ordering::SeqCst), 1);
}

#[test]
fn an_aborted_fetch_holds_its_worker_until_it_ends() {
    let (release, hold) = mpsc::channel();
    let store = Store {
        fetches: Arc::default(),
        fail: false,
        hold: Some(Mutex::new(hold)),
    };
    let (req, fetches) = request(store);
    let cache = BlockCache::new(1024 * 1024);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(1);

    assert!(prefetcher.request(req(), &cache));
    until(|| fetches.load(Ordering::SeqCst) == 1);
    prefetcher.abort();
    // Still downloading: the lookahead must not start another on top.
    assert_eq!(prefetcher.active_worker_threads(), 1);
    assert!(!prefetcher.request(req(), &cache));
    let _ = release.send(());
    until(|| prefetcher.active_worker_threads() == 0);
    assert!(
        prefetcher.poll().is_empty(),
        "an aborted result is never read"
    );
}

#[test]
fn active_bytes_retire_on_drain() {
    let (release, hold) = mpsc::channel();
    let store = Store {
        fetches: Arc::default(),
        fail: false,
        hold: Some(Mutex::new(hold)),
    };
    let (req, _) = request(store);
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(1);
    let cache = BlockCache::new(1024 * 1024);

    assert!(prefetcher.request(req(), &cache));
    assert_eq!(prefetcher.pending_count(), 1);
    assert_eq!(prefetcher.total_bytes(), 40); // 10 elements * 4 bytes
    assert_eq!(prefetcher.pending_bytes(), 40);

    // Draining while the worker is held leaves the request pending.
    assert!(prefetcher.poll().is_empty());
    assert_eq!(prefetcher.total_bytes(), 40);

    // The only worker is busy: the lookahead waits, a view's block does not.
    let other = || {
        let mut r = req();
        r.slice = SliceRequest::new("temperature", vec![DimensionSelection::range(10, 20)]);
        r
    };
    assert!(!prefetcher.request(other(), &cache));
    assert!(prefetcher.request_now(other(), &cache));

    // Abort clears every pending request and its bytes.
    prefetcher.abort();
    assert_eq!(prefetcher.pending_count(), 0);
    assert_eq!(prefetcher.total_bytes(), 0);
    assert_eq!(prefetcher.completed_bytes(), 0);

    let _ = release.send(());
    let _ = release.send(());
}
