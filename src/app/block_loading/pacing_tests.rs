//! Prefetch pacing: worker counts, lookahead and the chunks queued while
//! playing or paused.

use super::pacing::*;

const MB: usize = 1024 * 1024;
const GB: usize = 1024 * MB;

#[test]
fn concurrency_scales_down_with_chunk_size_but_keeps_three() {
    assert_eq!(adaptive_concurrency(MB, 16), 16);
    assert_eq!(adaptive_concurrency(4 * MB, 16), 6);
    assert_eq!(adaptive_concurrency(16 * MB, 16), 3);
    assert_eq!(adaptive_concurrency(500 * MB, 16), 3);
    // The user's limit wins.
    assert_eq!(adaptive_concurrency(MB, 4), 4);
    assert_eq!(adaptive_concurrency(4 * MB, 2), 2);
}

#[test]
fn playing_lookahead_counts_steps_and_keeps_two_chunks() {
    // One step per chunk: the window's 32 steps.
    assert_eq!(playing_lookahead(32, 1, 4 * MB, GB), 32);
    // 24 steps per ~100 MB chunk: two chunks, not one.
    assert_eq!(playing_lookahead(32, 24, 100 * MB, GB), 2);
    // A small cache bounds it to half the cache, current chunk included...
    assert_eq!(playing_lookahead(32, 1, 10 * MB, 100 * MB), 5);
    // ...but the current chunk and the next are always queued.
    assert_eq!(playing_lookahead(32, 1, 100 * MB, 100 * MB), 2);
}

#[test]
fn range_budget_is_three_quarters_of_the_cache() {
    assert_eq!(range_budget(4 * MB, GB), 192);
    assert_eq!(range_budget(GB, GB), 1);
}

#[test]
fn playing_queues_the_lookahead_and_wraps_when_looping() {
    assert_eq!(playing_chunks(3, 20, 2, true), vec![3, 4]);
    assert_eq!(playing_chunks(19, 20, 3, false), vec![19, 20]);
    assert_eq!(playing_chunks(19, 20, 3, true), vec![19, 20, 0]);
    // Never wraps onto the current chunk.
    assert_eq!(playing_chunks(1, 2, 5, true), vec![1, 2, 0]);
    assert_eq!(playing_chunks(0, 2, 5, true), vec![0, 1, 2]);
}

#[test]
fn paused_queues_the_whole_range_from_the_current_chunk() {
    assert_eq!(paused_chunks(2, (0, 5), 99, 100), vec![2, 3, 4, 5, 0, 1]);
    assert_eq!(paused_chunks(0, (0, 3), 99, 100), vec![0, 1, 2, 3]);
}

#[test]
fn a_paused_one_chunk_range_still_queues_the_next_two() {
    // The default range is one chunk: stepping past it stays warm.
    assert_eq!(paused_chunks(0, (0, 0), 99, 100), vec![0, 1, 2]);
}

#[test]
fn paused_range_stays_within_the_budget() {
    assert_eq!(paused_chunks(2, (0, 50), 99, 4), vec![2, 3, 4, 5]);
    assert_eq!(paused_chunks(2, (0, 50), 99, 0), vec![2]);
}

#[test]
fn paused_outside_the_range_queues_the_steps_after_the_current_one() {
    assert_eq!(paused_chunks(10, (0, 3), 99, 100), vec![10, 11, 12]);
    assert_eq!(paused_chunks(98, (0, 3), 99, 100), vec![98, 99]);
}
