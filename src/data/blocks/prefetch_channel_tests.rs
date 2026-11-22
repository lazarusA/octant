//! The result channel never blocks a sender (the browser has one thread).

use super::{BlockPrefetcher, PrefetchResult};
use crate::data::BlockCacheKey;

/// After an abort, more results than twice the workers can finish before
/// the next poll (`request_now` skips the worker limit): none may block.
#[test]
fn results_past_twice_the_workers_never_block_after_abort() {
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(2);
    prefetcher.abort();
    for i in 0..7 {
        let key = BlockCacheKey {
            source_id: "s".into(),
            variable_name: format!("v{i}"),
            selections: Vec::new(),
        };
        let result = Err("x".to_string());
        assert!(
            prefetcher
                .tx
                .send(PrefetchResult {
                    key,
                    result,
                    unrequested: false,
                    request: 0,
                })
                .is_ok()
        );
    }
}

#[test]
fn a_result_sent_before_its_cancel_still_comes_back_unrequested() {
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(2);
    let key = BlockCacheKey {
        source_id: "s".into(),
        variable_name: "v".into(),
        selections: Vec::new(),
    };
    prefetcher.pending.insert(key.clone());
    // The task sent while its cancel flag was still down.
    let sent = prefetcher.tx.send(PrefetchResult {
        key: key.clone(),
        result: Err("x".to_string()),
        unrequested: false,
        request: 0,
    });
    assert!(sent.is_ok());
    assert!(prefetcher.cancel(&key));
    let results = prefetcher.poll();
    assert_eq!(results.len(), 1);
    assert!(results[0].unrequested, "nobody waits on it");
}

#[test]
fn a_result_sent_before_its_cancel_never_answers_the_next_request() {
    let mut prefetcher = BlockPrefetcher::with_max_concurrent_threads(2);
    let key = BlockCacheKey {
        source_id: "s".into(),
        variable_name: "v".into(),
        selections: Vec::new(),
    };
    // The first request's task failed and sent while its flag was down.
    let first = prefetcher.track(key.clone());
    let sent = prefetcher.tx.send(PrefetchResult {
        key: key.clone(),
        result: Err("x".to_string()),
        unrequested: false,
        request: first.request,
    });
    assert!(sent.is_ok());
    assert!(prefetcher.cancel(&key));
    // Asked again before the poll.
    let _second = prefetcher.track(key.clone());
    let results = prefetcher.poll();
    assert_eq!(results.len(), 1);
    assert!(results[0].unrequested, "the old answer is nobody's");
    assert!(prefetcher.is_pending(&key), "the new request still waits");
}
