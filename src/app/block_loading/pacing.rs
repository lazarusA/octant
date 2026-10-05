//! Adaptive concurrency pacing and lookahead calculations for prefetching.

use crate::data::SliceRequest;

/// Calculates estimated bytes per chunk slice without cloning or heap allocation.
pub fn estimate_chunk_bytes(base_req: &SliceRequest, anim_dim: usize, cs: usize) -> usize {
    let elements = base_req
        .selections
        .iter()
        .enumerate()
        .map(|(d, sel)| {
            if d == anim_dim {
                cs.max(1)
            } else {
                let (start, end) = sel.bounds();
                end.saturating_sub(start).max(1)
            }
        })
        .try_fold(1usize, |acc, count| acc.checked_mul(count))
        .unwrap_or(usize::MAX);
    elements.saturating_mul(4).max(1)
}

const MB: usize = 1024 * 1024;
const CHUNK_SMALL_MB: usize = 2 * MB;
const CHUNK_MEDIUM_MB: usize = 8 * MB;
const CHUNK_LARGE_MB: usize = 32 * MB;

/// Dynamically scales worker concurrency limit according to chunk footprint.
pub fn adaptive_concurrency(chunk_bytes: usize, user_max_threads: usize) -> usize {
    let target = match chunk_bytes {
        0..=CHUNK_SMALL_MB => user_max_threads, // <= 2 MB: full thread pool
        c if c <= CHUNK_MEDIUM_MB => 6,         // 2 MB - 8 MB
        c if c <= CHUNK_LARGE_MB => 3,          // 8 MB - 32 MB
        _ => 2,                                 // > 32 MB: prioritize bandwidth
    };
    target.min(user_max_threads).max(1)
}

/// Dynamically scales lookahead chunk count based on memory budget.
pub fn adaptive_lookahead(
    window_size: usize,
    cs: usize,
    chunk_bytes: usize,
    cache_max_bytes: usize,
) -> usize {
    let budget_chunks = (cache_max_bytes / 4 / chunk_bytes.max(1)).max(2);
    let raw_chunks = window_size / cs.max(1);
    raw_chunks.clamp(1, budget_chunks).max(1)
}

/// Parameters for dispatching a sequence of candidate lookahead chunks.
pub struct PrefetchDispatchParams<'a> {
    pub cs: usize,
    pub full_extent: usize,
    pub anim_dim: usize,
    pub max_concurrent: usize,
    pub base_req: &'a SliceRequest,
    pub store_handle: &'a crate::data::StoreHandle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_concurrency_scaling() {
        assert_eq!(adaptive_concurrency(1_000_000, 16), 16);
        assert_eq!(adaptive_concurrency(4_000_000, 16), 6);
        assert_eq!(adaptive_concurrency(16_000_000, 16), 3);
        assert_eq!(adaptive_concurrency(50_000_000, 16), 2);
        // User thread limit respected
        assert_eq!(adaptive_concurrency(1_000_000, 4), 4);
        assert_eq!(adaptive_concurrency(4_000_000, 2), 2);
    }

    #[test]
    fn test_adaptive_lookahead_budgeting() {
        // Large cache, small chunks -> full window
        assert_eq!(adaptive_lookahead(32, 1, 100_000, 512 * 1024 * 1024), 32);
        // Huge chunks -> bounded by budget
        assert_eq!(
            adaptive_lookahead(32, 1, 50 * 1024 * 1024, 128 * 1024 * 1024),
            2
        );
    }
}
