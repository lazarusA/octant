//! What opening real consolidated stores costs, timed by hand:
//! `cargo test --release --lib array_opens_timed -- --ignored --nocapture`
//! opens the arrays of TESSERA's `global_rgb` pyramid (root `zarr.json` of
//! 451 nodes); `dataset_opens_timed` opens TESSERA, an EOPF Sentinel-2
//! store (deep groups) and Earthmover's ERA5 Icechunk repository (desktop
//! Icechunk reads through the same store) and reads a first block, as the
//! app does.

use std::time::{Duration, Instant};

use super::store::ZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

const TESSERA: &str = "https://tessera-embeddings.s3.amazonaws.com/v1.1/cambridge.zarr/";
const EOPF: &str = "https://s3.explorer.eopf.copernicus.eu/esa-zarr-sentinel-explorer-fra/tests-output/sentinel-2-l2a/S2C_MSIL2A_20260120T084301_N0511_R064_T36UXA_20260120T122910.zarr/measurements/reflectance";
const ERA5: &str = "https://earthmover-icechunk-era5.s3.us-east-1.amazonaws.com/icechunkV2";
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

/// A first block of `variable` (of `shape`): its first index along every
/// dimension but the last two, of which at most 256 x 256.
fn first_block(variable: &str, shape: &[u64]) -> SliceRequest {
    let rank = shape.len();
    let selections = (shape.iter().enumerate())
        .map(|(i, &len)| match i + 2 >= rank {
            true => DimensionSelection::Range {
                start: 0,
                end: (len as usize).min(256),
            },
            false => DimensionSelection::Index(0),
        })
        .collect();
    SliceRequest {
        variable: variable.into(),
        selections,
    }
}

/// The block store of `url`, Icechunk for ERA5, else Zarr.
fn open_store(url: &str) -> Box<dyn BlockStore> {
    match url == ERA5 {
        true => {
            Box::new(crate::data::backends::icechunk::IcechunkBlockStore::open(url).expect("open"))
        }
        false => Box::new(ZarrBlockStore::open_remote(url).expect("open")),
    }
}

/// Per store, opening the dataset (`inspect`) and then the first block of
/// its first variable of at least two dimensions; each run opens the store
/// afresh.
#[test]
#[ignore = "network"]
fn dataset_opens_timed() {
    let median = |mut v: Vec<Duration>| {
        v.sort();
        v[v.len() / 2]
    };
    eprintln!("store     inspect  first block  variables  (median of {RUNS} runs)");
    for (name, url) in [("TESSERA", TESSERA), ("EOPF", EOPF), ("ERA5", ERA5)] {
        let (mut inspects, mut blocks, mut count) = (Vec::new(), Vec::new(), 0);
        for _ in 0..RUNS {
            let store = open_store(url);
            let start = Instant::now();
            let meta = store.inspect().expect("inspect");
            inspects.push(start.elapsed());
            count = meta.variables.len();
            let var = (meta.variables.iter())
                .find(|v| v.shape.len() >= 2)
                .expect("a 2D variable");
            let start = Instant::now();
            store
                .fetch_block(&first_block(&var.name, &var.shape))
                .expect("block");
            blocks.push(start.elapsed());
        }
        let (inspect, block) = (median(inspects), median(blocks));
        eprintln!("{name:<8} {inspect:>8.2?} {block:>12.2?} {count:>10}");
    }
}

/// Metadata read while opening TESSERA and the first block of levels 9 to
/// 4, and ERA5 and the first block of its first variable: requests and
/// bytes, the same on any network. Prints each request, `zarr.json` files
/// only, after its store's name.
#[test]
#[ignore = "network"]
fn metadata_reads_counted() {
    use std::sync::{Arc, Mutex};
    use zarrs::storage::storage_adapter::usage_log::UsageLogStorageAdapter;
    for (name, url) in [("TESSERA", TESSERA), ("ERA5", ERA5)] {
        let log = Arc::new(Mutex::new(Vec::<u8>::new()));
        let remote = match url == ERA5 {
            true => crate::data::backends::icechunk::build_sync_icechunk_store(url),
            false => super::storage::build_sync_store(url),
        };
        let logged = UsageLogStorageAdapter::new(remote.expect("store"), log.clone(), String::new);
        let store =
            super::generic::GenericZarrBlockStore::new(Arc::new(logged), url, "zarr", "Zarr");
        let meta = store.inspect().expect("inspect");
        let requests = match url == ERA5 {
            true => (meta.variables.iter().find(|v| v.shape.len() >= 2))
                .map(|v| vec![first_block(&v.name, &v.shape)])
                .unwrap_or_default(),
            false => LEVELS.map(|level| chunk(level, 0)).to_vec(),
        };
        for request in &requests {
            store.fetch_block(request).expect("block");
        }
        let log = log.lock().unwrap_or_else(|p| p.into_inner());
        for line in String::from_utf8_lossy(&log).lines() {
            if line.contains("zarr.json") {
                eprintln!("READ {name} {line}");
            }
        }
    }
}
