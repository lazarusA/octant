//! Brick read measurements: per level of a few
//! multiscale stores, the time to read one brick (64^3 interior, as
//! `fetch.rs` reads it) and one with its apron (66^3, as before the aprons
//! came from neighbours), the chunks each touches, and eight bricks read at
//! once (`IN_FLIGHT`). Reads go straight to the store, decode included, at
//! a different brick each time so nothing is served from a cache.
//! Run by hand with
//! `cargo test --release --lib brick_fetch_timed -- --ignored --nocapture --test-threads 1`.

use std::ops::Range;
use std::time::{Duration, Instant};

use crate::app::layers::LayerId;
use crate::app::multiscale::to_level;
use crate::app::{OctantApp, StoreKind};
use crate::data::multiscale::bricks::{BRICK, BrickKey, Bricks};
use crate::data::{BlockRequest, DataSource, Dataset, SourceFactory};

const STORES: [(&str, &str); 4] = [
    (
        "beechnut",
        "https://d39zu0xtgv0613.cloudfront.net/beechnut.ome.zarr",
    ),
    (
        "janelia hela-2",
        "https://janelia-cosem-datasets.s3.amazonaws.com/jrc_hela-2/jrc_hela-2.zarr/recon-1/em/fibsem-uint8",
    ),
    (
        "kingsnake",
        "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/kingsnake.ome.zarr",
    ),
    (
        "woodbranch",
        "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/woodbranch.ome.zarr",
    ),
];

/// The store at `url` open, its multiscale variable selected; `None` when
/// it cannot be read.
fn opened(url: &str) -> Option<OctantApp> {
    let mut app = OctantApp::default();
    let kind = StoreKind::RemoteZarr;
    let source_id = StoreKind::make_source_id(kind, url);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), url, "Store");
    let handle = SourceFactory::open(source.clone()).ok()?;
    let meta = handle.inspect().ok()?;
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_kind = kind;
    app.selected.store_target = url.to_string();
    app.load_new_metadata(meta.clone());
    let idx = meta.variables.iter().position(|v| v.multiscale.is_some())?;
    let var = meta.variables[idx].clone();
    app.selected.variable_idx = idx;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    Some(app)
}

/// The request reading `window` (cells of level `level` along x, y, z).
fn request(app: &OctantApp, level: usize, window: &[Range<u64>; 3]) -> Option<BlockRequest> {
    let var = app.selected.variable_info()?;
    let (x, y, z) = var.resolve_spatial_dim_indices(&app.selected.dim_config);
    let dims = [x?, y?, z?];
    let mut sel = to_level(&app.selected, level)?;
    for (i, w) in window.iter().enumerate() {
        let (start, end) = (w.start as usize, w.end.saturating_sub(1) as usize);
        sel.set_range(dims[i], (start, end), start);
    }
    let read = app.level_read(LayerId::BASE, sel)?;
    Some(BlockRequest::new(read.handle, read.slice))
}

/// Wall time of reading every request at once, and how many failed.
fn timed(requests: Vec<BlockRequest>) -> (Duration, usize) {
    let start = Instant::now();
    let threads: Vec<_> = requests
        .into_iter()
        .map(|r| std::thread::spawn(move || r.store.fetch(&r.slice).is_err()))
        .collect();
    let failed = threads
        .into_iter()
        .map(|t| t.join())
        .filter(|r| !matches!(r, Ok(false)))
        .count();
    (start.elapsed(), failed)
}

/// Chunks of `chunk` cells a window `w` touches along one axis.
fn chunks_along(w: &Range<u64>, chunk: u64) -> u64 {
    let c = chunk.max(1);
    (w.end.saturating_sub(1)) / c - w.start / c + 1
}

/// Prints one level's measurements.
fn measure_level(app: &OctantApp, bricks: &Bricks, level: usize, chunk: &[u64; 3]) {
    let counts = bricks.counts(level);
    // Bricks near the middle, one per measurement.
    let at = |dx: u32| BrickKey {
        level: level as u8,
        b: std::array::from_fn(|i| {
            (counts[i] / 2 + if i == 0 { dx } else { 0 }).min(counts[i] - 1)
        }),
    };
    let Some(inner) = bricks.window(at(0)) else {
        return;
    };
    let apron = bricks.window(at(1)).map(|w| {
        std::array::from_fn::<_, 3, _>(|i| {
            let start = w[i].start.saturating_sub(1);
            start..w[i].end + 1
        })
    });
    let Some(apron) = apron else {
        return;
    };
    let chunks = |w: &[Range<u64>; 3]| {
        (0..3)
            .map(|i| chunks_along(&w[i], chunk[i]))
            .product::<u64>()
    };
    let one = |w: &[Range<u64>; 3]| timed(request(app, level, w).into_iter().collect()).0;
    let (t_inner, t_apron) = (one(&inner), one(&apron));
    // Distinct bricks only: at small levels the offsets run into the edge.
    let mut keys: Vec<BrickKey> = (2..10).map(at).collect();
    keys.dedup();
    keys.retain(|k| *k != at(0) && *k != at(1));
    let eight: Vec<BlockRequest> = keys
        .into_iter()
        .filter_map(|k| bricks.window(k))
        .filter_map(|w| request(app, level, &w))
        .collect();
    let n = eight.len();
    let (t_eight, failed) = timed(eight);
    eprintln!(
        "  level {level} ({} x {} x {} bricks): one brick {:.0} ms, {} chunks; with apron {:.0} ms, \
         {} chunks; {n} at once {:.0} ms ({:.0} ms per brick, {failed} failed)",
        counts[0],
        counts[1],
        counts[2],
        t_inner.as_secs_f64() * 1e3,
        chunks(&inner),
        t_apron.as_secs_f64() * 1e3,
        chunks(&apron),
        t_eight.as_secs_f64() * 1e3,
        t_eight.as_secs_f64() * 1e3 / n.max(1) as f64,
    );
}

#[test]
#[ignore = "downloads from four stores: run by hand in release"]
fn brick_fetch_timed() {
    for (name, url) in STORES {
        let Some(app) = opened(url) else {
            eprintln!("{name}: could not open {url}");
            continue;
        };
        let Some(var) = app.selected.variable_info().cloned() else {
            continue;
        };
        let (Some(set), (Some(x), Some(y), Some(z))) = (
            var.multiscale.as_ref().map(|r| r.set.clone()),
            var.resolve_spatial_dim_indices(&app.selected.dim_config),
        ) else {
            eprintln!("{name}: not a 3D stack");
            continue;
        };
        let dims = [x, y, z];
        let chunk = dims.map(|d| var.chunk_shape.get(d).copied().unwrap_or(BRICK));
        eprintln!(
            "{name}: {} levels, read unit {chunk:?} (x, y, z)",
            set.len()
        );
        let bricks = Bricks { set: &set, dims };
        for level in 0..set.len() {
            measure_level(&app, &bricks, level, &chunk);
        }
    }
}
