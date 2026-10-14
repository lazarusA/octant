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

/// Lookahead workers for chunks of `chunk_bytes`: the user's limit for small
/// chunks, fewer for larger ones so they share the bandwidth, never fewer
/// than three so large chunks still overlap.
pub fn adaptive_concurrency(chunk_bytes: usize, user_max_threads: usize) -> usize {
    let target = match chunk_bytes {
        0..=CHUNK_SMALL_MB => user_max_threads,
        c if c <= CHUNK_MEDIUM_MB => 6,
        _ => 3,
    };
    target.min(user_max_threads).max(1)
}

/// Chunks of `chunk_bytes` that `bytes` of the block cache hold, at least one.
fn cache_chunks(bytes: usize, chunk_bytes: usize) -> usize {
    (bytes / chunk_bytes.max(1)).max(1)
}

/// Chunks of `cs` steps queued ahead of the current one while playing: the
/// chunks holding `window_size` steps, at least two so the next chunk is
/// always on its way, within half the cache so the lookahead never evicts
/// the step on screen.
pub fn playing_lookahead(
    window_size: usize,
    cs: usize,
    chunk_bytes: usize,
    cache_max_bytes: usize,
) -> usize {
    let wanted = window_size.div_ceil(cs.max(1)).max(2);
    wanted.min(cache_chunks(cache_max_bytes / 2, chunk_bytes))
}

/// Chunks of a paused selected range prefetched at most: three quarters of
/// the cache, so the range never evicts the step on screen.
pub fn range_budget(chunk_bytes: usize, cache_max_bytes: usize) -> usize {
    cache_chunks(cache_max_bytes / 4 * 3, chunk_bytes)
}

/// The chunks to queue while playing from chunk `current`: it and the next
/// `lookahead`, wrapping to the first ones when `looping` reaches `last`.
pub fn playing_chunks(current: usize, last: usize, lookahead: usize, looping: bool) -> Vec<usize> {
    let mut indices: Vec<usize> = (current..=(current + lookahead).min(last)).collect();
    if looping && current + lookahead > last {
        let wrap_end = (current + lookahead - last - 1).min(current.saturating_sub(1));
        indices.extend((0..=wrap_end).filter(|c| *c < current));
    }
    indices
}

/// The chunks to queue while paused at chunk `current` with the chunks
/// `first..=last` selected: the current one, the rest of the range ahead of
/// it, then the range before it, at most `budget`. A current chunk outside
/// the range queues it and the two after it (`max_chunk` the dataset's last).
pub fn paused_chunks(
    current: usize,
    (first, last): (usize, usize),
    max_chunk: usize,
    budget: usize,
) -> Vec<usize> {
    if current < first || current > last {
        return (current..=(current + 2).min(max_chunk)).collect();
    }
    let ahead = current..=last;
    let behind = first..current;
    ahead.chain(behind).take(budget.max(1)).collect()
}

/// Parameters for dispatching a sequence of candidate lookahead chunks.
pub struct PrefetchDispatchParams<'a> {
    pub cs: usize,
    pub full_extent: usize,
    pub anim_dim: usize,
    pub max_concurrent: usize,
    pub base_req: &'a SliceRequest,
    pub store_handle: &'a crate::data::StoreHandle,
    pub source_id: &'a str,
}
