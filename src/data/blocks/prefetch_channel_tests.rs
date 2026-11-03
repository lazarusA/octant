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
        assert!(prefetcher.tx.send(PrefetchResult { key, result }).is_ok());
    }
}
