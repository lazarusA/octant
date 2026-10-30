//! What opening an array of a real consolidated store costs: TESSERA's
//! `global_rgb` pyramid (root `zarr.json` of 451 nodes), timed by hand with
//! `cargo test --release --lib array_opens_timed -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use super::store::ZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

const TESSERA: &str = "https://tessera-embeddings.s3.amazonaws.com/v1.1/cambridge.zarr/";
const LEVELS: [u32; 6] = [9, 8, 7, 6, 5, 4];
const RUNS: usize = 3;

/// One 512 x 512 chunk of `global_rgb/{level}/rgb`, the `row`th down.
fn chunk(level: u32, row: usize) -> SliceRequest {
    SliceRequest {
        variable: format!("global_rgb/{level}/rgb"),
        selections: vec![
            DimensionSelection::Range {
                start: row * 512,
                end: (row + 1) * 512,
            },
            DimensionSelection::Range { start: 0, end: 512 },
            DimensionSelection::Range { start: 0, end: 4 },
        ],
    }
}

fn timed(store: &ZarrBlockStore, request: &SliceRequest) -> Duration {
    let start = Instant::now();
    store.fetch_block(request).expect("block");
    start.elapsed()
}

/// Per level, the first block (the array's open and one chunk) and a
/// second chunk of the opened array; their difference is what the open
/// costs. Each run opens the store afresh.
#[test]
#[ignore = "network"]
fn array_opens_timed() {
    let mut opens = vec![Vec::new(); LEVELS.len()];
    let mut totals = Vec::new();
    for _ in 0..RUNS {
        let store = ZarrBlockStore::open_remote(TESSERA).expect("open");
        let run = Instant::now();
        for (i, level) in LEVELS.into_iter().enumerate() {
            let first = timed(&store, &chunk(level, 0));
            let second = timed(&store, &chunk(level, 1));
            opens[i].push((first, second));
        }
        totals.push(run.elapsed());
    }
    let median = |mut v: Vec<Duration>| {
        v.sort();
        v[v.len() / 2]
    };
    eprintln!("level  first block  second block  (median of {RUNS} runs)");
    for (level, runs) in LEVELS.iter().zip(opens) {
        let first = median(runs.iter().map(|r| r.0).collect());
        let second = median(runs.iter().map(|r| r.1).collect());
        eprintln!("{level:>5}  {first:>11.2?}  {second:>12.2?}");
    }
    eprintln!("all levels, 2 blocks each: {:.2?}", median(totals));
}
