//! Prefetch and playback from remote catalog stores, through the desktop
//! loading path: a range of steps plotted paused must arrive whole (timing
//! the first block and the range), and ERA5 `u10` with `sst` overlaid plays
//! with and without a buffer. Run by hand with
//! `cargo test --release --lib prefetch_network -- --ignored --nocapture --test-threads 1`.

use std::time::{Duration, Instant};

use crate::app::layers::LayerId;
use crate::app::{AnimationRole, OctantApp, SpatialRole, StoreKind};
use crate::data::{DataSource, Dataset, DatasetMetadata, SourceFactory, VariableInfo};
use crate::plots::PlotType;

/// Bytes a test range may download at most.
const RANGE_BYTES: u64 = 512 * 1024 * 1024;
const ERA5: &str = "https://earthmover-icechunk-era5.s3.us-east-1.amazonaws.com/icechunkV2";

/// An app with the remote store `target` open and its metadata loaded.
fn open_remote(target: &str, kind: StoreKind) -> (OctantApp, DatasetMetadata) {
    let mut app = OctantApp::default();
    let source_id = StoreKind::make_source_id(kind, target);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), target, "Store");
    let handle = SourceFactory::open(source.clone()).expect("open store");
    let meta = handle.inspect().expect("inspect store");
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_kind = kind;
    app.selected.store_target = target.to_string();
    app.load_new_metadata(meta.clone());
    app.layout.show_hero = false;
    (app, meta)
}

/// Bytes of one step of `var` along `time` (every other dimension whole).
fn step_bytes(var: &VariableInfo, time: usize) -> u64 {
    let cells: u64 = var
        .shape
        .iter()
        .enumerate()
        .filter(|&(d, _)| d != time)
        .map(|(_, &n)| n)
        .product();
    cells.saturating_mul(4)
}

/// Selects variable `idx` as a heatmap animated along `time`, its range the
/// first `steps` steps, and plots it paused.
fn plot_range(app: &mut OctantApp, idx: usize, time: usize, steps: usize) {
    let var = app.selected.metadata.as_ref().expect("metadata").variables[idx].clone();
    app.selected.variable_idx = idx;
    app.selected.plot_type = PlotType::Heatmap;
    crate::ui::variables_panel::init_variable_dimension_defaults(app, &var);
    let config = &mut app.selected.dim_config[time];
    config.spatial = SpatialRole::None;
    config.animation = AnimationRole::Animated;
    config.active = true;
    config.range = (0, steps - 1);
    app.selected.animated_dim = Some(time);
    app.selected.spatial_dims.retain(|&d| d != time);
    app.selected.dim_ranges[time] = (0, steps - 1);
    app.playback.current_timestep = 0;
    app.plot_selection();
}

/// Plots the first steps of `meta`'s variable `idx` along `time` (within
/// [`RANGE_BYTES`]), waits for the range and asserts every step arrived.
fn check_range(app: &mut OctantApp, label: &str, idx: usize, time: usize) {
    let var = app.selected.metadata.as_ref().expect("metadata").variables[idx].clone();
    let extent = var.shape[time] as usize;
    let cs = var.chunk_shape.get(time).copied().unwrap_or(1).max(1) as usize;
    let fit = (RANGE_BYTES / step_bytes(&var, time).max(1)) as usize;
    let steps = (4 * cs).max(24).min(fit.max(cs)).min(extent).max(1);
    eprintln!(
        "{label}: '{}' shape {:?} chunks {:?}, {steps} steps of {:.1} MB",
        var.name,
        var.shape,
        var.chunk_shape,
        step_bytes(&var, time) as f64 / 1e6
    );

    let started = Instant::now();
    plot_range(app, idx, time, steps);
    let deadline = started + Duration::from_secs(300);
    let mut first = None;
    while app.block_prefetcher.pending_count() > 0 {
        assert!(
            Instant::now() < deadline,
            "{label}: the range never arrived"
        );
        app.poll_block_prefetch_results();
        if first.is_none() && app.layers.base.data.matrix.is_some() {
            first = Some(started.elapsed());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let missing: Vec<usize> = (0..steps)
        .filter(|&t| !app.layer_step_resident(LayerId::BASE, t))
        .collect();
    eprintln!(
        "{label}: first block {:?}, {steps} steps in {:?}, {} blocks cached",
        first,
        started.elapsed(),
        app.block_cache.cached_count()
    );
    assert!(
        missing.is_empty(),
        "{label}: steps never loaded {missing:?}"
    );
}

/// The time dimension of `var`, when its steps are worth playing.
fn time_dim(var: &VariableInfo) -> Option<usize> {
    let t = var
        .dimension_names
        .iter()
        .position(|d| crate::data::coordinates::naming::is_animated_time_name(d))?;
    (var.shape.len() > 2 && var.shape[t] > 1).then_some(t)
}

/// The gridded variable of `meta` named `name` (or ending in `/name`) with
/// the smallest time chunks: the one whose steps download alone.
fn variable_named(meta: &DatasetMetadata, name: &str) -> Option<(usize, usize)> {
    meta.variables
        .iter()
        .enumerate()
        .filter(|(_, v)| v.name == name || v.name.ends_with(&format!("/{name}")))
        .filter_map(|(i, v)| Some((i, time_dim(v)?)))
        .min_by_key(|&(i, t)| meta.variables[i].chunk_shape.get(t).copied())
}

/// The smallest image level of `meta` (no label images) with a time axis.
fn smallest_level(meta: &DatasetMetadata) -> Option<(usize, usize)> {
    meta.variables
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_label_image())
        .filter_map(|(i, v)| Some((i, time_dim(v)?)))
        .min_by_key(|&(i, _)| meta.variables[i].shape.iter().product::<u64>())
}

#[test]
#[ignore = "reads remote stores"]
fn prefetch_network_era5_t2m_range() {
    let (mut app, meta) = open_remote(ERA5, StoreKind::RemoteIcechunk);
    for v in meta.variables.iter().filter(|v| v.name.ends_with("t2m")) {
        eprintln!(
            "t2m variable '{}' dims {:?} chunks {:?}",
            v.name, v.dimension_names, v.chunk_shape
        );
    }
    let (idx, time) = variable_named(&meta, "t2m").expect("a t2m variable with time");
    check_range(&mut app, "ERA5 t2m", idx, time);
}

#[test]
#[ignore = "reads remote stores"]
fn prefetch_network_microscopy_time_lapses() {
    for key in ["idr0101-time-lapse", "idr0052-mitosis"] {
        let entry = crate::catalog::MICROSCOPY_CATALOG
            .iter()
            .find(|e| e.key == key)
            .expect("catalog entry");
        let (mut app, meta) = open_remote(entry.store, entry.store_kind);
        let (idx, time) = smallest_level(&meta).expect("a level with a t axis");
        check_range(&mut app, key, idx, time);
    }
}

/// Polls finished fetches and coordinates, as a frame does, until `done` (at most a minute).
fn wait(app: &mut OctantApp, what: &str, done: impl Fn(&OctantApp) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !done(app) {
        assert!(Instant::now() < deadline, "{what} never arrived");
        app.poll_block_prefetch_results();
        app.poll_coordinate_results();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// ERA5 `u10` plotted with `sst` overlaid, both at step 0; the overlay's id.
fn two_layer_app() -> (OctantApp, LayerId) {
    let (mut app, meta) = open_remote(ERA5, StoreKind::RemoteIcechunk);
    let index = |name: &str| meta.variables.iter().position(|v| v.name == name);
    let base = index("single/spatial/u10").expect("u10");
    app.selected.variable_idx = base;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &meta.variables[base]);
    app.plot_selection();
    wait(&mut app, "the base block", |a| {
        a.layers.base.data.matrix.is_some()
    });
    let id = app
        .add_overlay(index("single/spatial/sst").expect("sst"))
        .expect("overlay");
    wait(&mut app, "the overlay block", |a| {
        a.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
    (app, id)
}

/// Plays at 10 fps for 30 s buffering `buffer_secs`, as the frame timer does;
/// logs the steps reached, the stalls (pauses over two frames) and the
/// longest run of steps without one, and the frames the overlay lagged.
fn play(app: &mut OctantApp, id: LayerId, buffer_secs: f32) {
    app.playback.playback_fps = 10.0;
    app.playback.buffer_secs = buffer_secs;
    app.toggle_playback();
    let frame = Duration::from_millis(100);
    let started = Instant::now();
    let (mut last, mut last_move) = (app.playback.current_timestep, Instant::now());
    let (mut stalls, mut run, mut longest, mut lagging, mut frames) = (0, 0, 0, 0, 0);
    while started.elapsed() < Duration::from_secs(30) {
        let now = Instant::now();
        if now.duration_since(app.playback.last_step_time) >= frame {
            app.advance_playback(now);
        }
        app.poll_block_prefetch_results();
        let step = app.playback.current_timestep;
        if step != last {
            if last_move.elapsed() > frame * 2 {
                (stalls, run) = (stalls + 1, 0);
            }
            run += 1;
            longest = longest.max(run);
            (last, last_move) = (step, Instant::now());
        }
        let overlay = app
            .layers
            .get(id)
            .and_then(|l| l.selection().dim_indices.first().copied());
        lagging += u32::from(overlay != Some(step));
        frames += 1;
        std::thread::sleep(Duration::from_millis(4));
    }
    eprintln!(
        "{buffer_secs} s buffer: step {last} in 30 s, {stalls} stalls, longest run {longest} steps, overlay behind {}% of frames",
        100 * lagging / frames.max(1)
    );
}

#[test]
#[ignore = "reads remote stores"]
fn prefetch_network_era5_two_layer_playback() {
    for buffer_secs in [0.0, 3.0] {
        let (mut app, id) = two_layer_app();
        play(&mut app, id, buffer_secs);
    }
}
