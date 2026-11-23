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

#[test]
fn only_chunked_remote_sources_count_their_stored_chunks() {
    use super::arrival_notices::stored_chunk_bytes;
    use crate::data::{DataSourceKind as Kind, VariableInfo};
    // ARCO-ERA5's pressure-level temperature: 37 levels per chunk.
    let era5 = VariableInfo {
        data_type: "float32".into(),
        shape: vec![1_323_648, 37, 721, 1440],
        chunk_shape: vec![1, 37, 721, 1440],
        ..VariableInfo::default()
    };
    assert_eq!(
        stored_chunk_bytes(&era5, &Kind::RemoteZarr, "https://h/s"),
        37 * 721 * 1440 * 4
    );
    assert_eq!(stored_chunk_bytes(&era5, &Kind::NetCdf, "https://h/s"), 0);
    assert_eq!(
        stored_chunk_bytes(&era5, &Kind::Procedural, "https://h/s"),
        0
    );
    // One chunk holding the whole array is the worst case for Zarr; NetCDF
    // and GRIB report that shape for any array, and so does Icechunk's
    // browser inspection when it cannot read an array's metadata.
    let one_chunk = VariableInfo {
        shape: vec![8760, 721, 1440],
        chunk_shape: vec![8760, 721, 1440],
        ..era5.clone()
    };
    assert_eq!(
        stored_chunk_bytes(&one_chunk, &Kind::RemoteZarr, "https://h/s"),
        8760 * 721 * 1440 * 4
    );
    assert_eq!(
        stored_chunk_bytes(&one_chunk, &Kind::RemoteIcechunk, "https://h/s"),
        0
    );
    assert_eq!(
        stored_chunk_bytes(&one_chunk, &Kind::NetCdf, "https://h/s"),
        0
    );
    // An `icechunk+` address opened as Zarr reads as Icechunk.
    let repo = "icechunk+s3://b/r";
    assert_eq!(stored_chunk_bytes(&one_chunk, &Kind::RemoteZarr, repo), 0);
    // The browser's local folders register as remote Zarr but read from disk.
    let local = "local://folder/era5.zarr";
    assert_eq!(stored_chunk_bytes(&era5, &Kind::RemoteZarr, local), 0);
    // A chunk running past a small array's edge counts only the array's part.
    let edge = VariableInfo {
        shape: vec![10, 4],
        chunk_shape: vec![100, 4],
        ..era5.clone()
    };
    assert_eq!(
        stored_chunk_bytes(&edge, &Kind::RemoteZarr, "https://h/s"),
        10 * 4 * 4
    );
}
