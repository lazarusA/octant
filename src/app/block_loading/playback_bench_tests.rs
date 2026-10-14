//! How fast ERA5 frames arrive and play, base plus overlay, through the
//! desktop path: raw block fetches at several concurrencies, then playback of
//! `single/spatial/u10` with `single/spatial/sst` overlaid. Run by hand with
//! `cargo test --release --lib playback_bench -- --ignored --nocapture --test-threads 1`.

use std::time::{Duration, Instant};

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::{SpatialRole, StoreKind};
use crate::data::{
    DataSource, Dataset, DatasetMetadata, DimensionSelection, SliceRequest, SourceFactory,
    StoreHandle,
};

const ERA5: &str = "https://earthmover-icechunk-era5.s3.us-east-1.amazonaws.com/icechunkV2";
const BASE: &str = "single/spatial/u10";
const OVERLAY: &str = "single/spatial/sst";

fn open() -> (StoreHandle, DatasetMetadata) {
    let kind = StoreKind::RemoteIcechunk;
    let source_id = StoreKind::make_source_id(kind, ERA5);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), ERA5, "Store");
    let handle = SourceFactory::open(source).expect("open store");
    let meta = handle.inspect().expect("inspect");
    (handle, meta)
}

/// One step of `var`, every latitude and longitude.
fn step_request(var: &str, t: usize) -> SliceRequest {
    let all = |end| DimensionSelection::Range { start: 0, end };
    SliceRequest::new(
        var,
        vec![
            DimensionSelection::Range {
                start: t,
                end: t + 1,
            },
            all(721),
            all(1440),
        ],
    )
}

#[test]
#[ignore = "reads remote stores"]
fn playback_bench_raw_fetch() {
    let (store, _) = open();
    // Warm up: the array opens and the coordinates load once.
    let warm = Instant::now();
    store.fetch(&step_request(BASE, 0)).expect("block");
    eprintln!("first block (open + fetch): {:?}", warm.elapsed());
    let one = Instant::now();
    for t in 1..=4 {
        store.fetch(&step_request(BASE, t)).expect("block");
    }
    eprintln!("sequential: {:?} per frame", one.elapsed() / 4);
    let mut first = 100;
    for threads in [1, 3, 6, 12, 24, 48] {
        let n = (threads * 3).max(8);
        let fps = rate(first, n, threads, |t| {
            store.fetch(&step_request(BASE, t)).expect("block");
        });
        eprintln!("{threads:>2} at once: {fps:5.1} frames/s ({n} frames)");
        first += n;
    }
}

/// Polls finished fetches and coordinates, as a frame does.
fn frame(app: &mut OctantApp) {
    app.poll_block_prefetch_results();
    app.poll_coordinate_results();
}

/// Frames until `done` holds (at most `secs`).
fn wait(app: &mut OctantApp, secs: u64, what: &str, done: impl Fn(&OctantApp) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !done(app) {
        assert!(Instant::now() < deadline, "{what} never happened");
        frame(app);
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// An app playing `BASE` with `OVERLAY` over it, both at step 0.
fn two_layer_app(store: StoreHandle, meta: &DatasetMetadata) -> (OctantApp, LayerId) {
    let mut app = OctantApp::default();
    let kind = StoreKind::RemoteIcechunk;
    let source_id = StoreKind::make_source_id(kind, ERA5);
    let source = store.source().clone();
    app.dataset_manager
        .add(Dataset::new(&source_id, source, store));
    app.selected.store_kind = kind;
    app.selected.store_target = ERA5.to_string();
    app.load_new_metadata(meta.clone());
    app.layout.show_hero = false;
    let index = |name: &str| meta.variables.iter().position(|v| v.name == name);
    let base = index(BASE).expect("base variable");
    let var = meta.variables[base].clone();
    app.selected.variable_idx = base;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    assert_eq!(app.selected.dim_config[0].spatial, SpatialRole::Z);
    app.plot_selection();
    wait(&mut app, 60, "the base block", |a| {
        a.layers.base.data.matrix.is_some()
    });
    let id = app
        .add_overlay(index(OVERLAY).expect("overlay variable"))
        .expect("overlay added");
    wait(&mut app, 60, "the overlay block", |a| {
        a.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
    (app, id)
}

/// The step overlay `id` shows.
fn overlay_step(app: &OctantApp, id: LayerId) -> usize {
    app.layers
        .get(id)
        .and_then(|l| l.selection().dim_indices.first().copied())
        .unwrap_or(usize::MAX)
}

/// Plays at `fps` for `secs`, buffering `buffer_secs` (0 shows each step as
/// it arrives); reports the steps reached, the stalls (pauses over two
/// frames), the longest run of steps without one, the frames the overlay
/// lagged and the slowest frame (projection on the UI thread).
fn play(app: &mut OctantApp, id: LayerId, fps: f32, buffer_secs: f32, secs: u64) {
    app.playback.playback_fps = fps;
    app.playback.buffer_secs = buffer_secs;
    app.toggle_playback();
    let frame_dur = Duration::from_secs_f32(1.0 / fps);
    let started = Instant::now();
    let (mut frames, mut lagging, mut slowest) = (0u32, 0u32, Duration::ZERO);
    let mut last_move = Instant::now();
    let (mut stalled, mut stalls, mut run, mut longest) = (Duration::ZERO, 0u32, 0u32, 0u32);
    let mut last_step = app.playback.current_timestep;
    while started.elapsed() < Duration::from_secs(secs) {
        let t0 = Instant::now();
        if t0.duration_since(app.playback.last_step_time) >= frame_dur {
            app.advance_playback(t0);
        }
        frame(app);
        slowest = slowest.max(t0.elapsed());
        frames += 1;
        let step = app.playback.current_timestep;
        if step != last_step {
            let gap = last_move.elapsed();
            if gap > frame_dur * 2 {
                stalled += gap - frame_dur;
                stalls += 1;
                run = 0;
            }
            run += 1;
            longest = longest.max(run);
            last_move = Instant::now();
            last_step = step;
        }
        lagging += u32::from(overlay_step(app, id) != step);
        std::thread::sleep(Duration::from_millis(4));
    }
    let elapsed = started.elapsed().as_secs_f64();
    eprintln!(
        "{fps:>4} fps, {buffer_secs} s buffer: reached step {}, {:.1} steps/s, {stalls} stalls ({stalled:.1?}), longest run {longest} steps, overlay behind {:.0}% of frames, slowest frame {:?}",
        app.playback.current_timestep,
        app.playback.current_timestep as f64 / elapsed,
        100.0 * f64::from(lagging) / f64::from(frames.max(1)),
        slowest,
    );
}

#[test]
#[ignore = "reads remote stores"]
fn playback_bench_two_layers() {
    let (store, meta) = open();
    for buffer_secs in [0.0, 3.0] {
        let (mut app, id) = two_layer_app(store.clone(), &meta);
        play(&mut app, id, 10.0, buffer_secs, 30);
    }
}

/// Runs `job(t)` for steps `first..first + n` with `threads` at once; frames/s.
fn rate(first: usize, n: usize, threads: usize, job: impl Fn(usize) + Sync) -> f64 {
    let next = std::sync::atomic::AtomicUsize::new(first);
    let started = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let t = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if t >= first + n {
                        return;
                    }
                    job(t);
                }
            });
        }
    });
    n as f64 / started.elapsed().as_secs_f64()
}

#[test]
#[ignore = "reads remote stores"]
fn playback_bench_stages() {
    use zarrs::storage::ReadableStorageTraits;
    let storage =
        crate::data::backends::icechunk::build_sync_icechunk_store(ERA5).expect("icechunk store");
    let readable: zarrs::storage::ReadableStorage = storage.clone();
    let array =
        crate::utils::metadata::open_or_instantiate_array_normalized(readable, &format!("/{BASE}"))
            .expect("array");
    eprintln!("codecs: {:?}", array.metadata().to_string());
    let key = |t: usize| array.chunk_key(&[t as u64, 0, 0]);
    let bytes = storage.get(&key(0)).expect("get").map_or(0, |b| b.len());
    eprintln!("chunk 0: {:.2} MB compressed", bytes as f64 / 1e6);

    let mut first = 1000;
    for threads in [1, 12] {
        let n = 12;
        let get = rate(first, n, threads, |t| {
            storage.get(&key(t)).expect("get");
        });
        first += n;
        let decode = rate(first, n, threads, |t| {
            let _: Vec<f32> = array.retrieve_chunk(&[t as u64, 0, 0]).expect("decode");
        });
        first += n;
        eprintln!("{threads:>2} at once: raw get {get:5.1}/s, get + decode {decode:5.1}/s");
    }
    let (store, _) = open();
    let full = rate(first, 12, 12, |t| {
        store.fetch(&step_request(BASE, t)).expect("block");
    });
    eprintln!("12 at once: fetch_block {full:5.1}/s");
}
