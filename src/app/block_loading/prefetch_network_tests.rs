//! Prefetch of a selected time range from remote catalog stores, through the
//! desktop loading path: plots a range of steps paused and checks that every
//! step arrives, logging how long the first block and the whole range took.
//! Run by hand with
//! `cargo test --lib prefetch_network -- --ignored --nocapture --test-threads 1`.

use std::time::{Duration, Instant};

use crate::app::layers::LayerId;
use crate::app::{AnimationRole, OctantApp, SpatialRole, StoreKind};
use crate::data::{DataSource, Dataset, DatasetMetadata, SourceFactory, VariableInfo};
use crate::plots::PlotType;

/// Bytes a test range may download at most.
const RANGE_BYTES: u64 = 512 * 1024 * 1024;

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
    let target = "https://earthmover-icechunk-era5.s3.us-east-1.amazonaws.com/icechunkV2";
    let (mut app, meta) = open_remote(target, StoreKind::RemoteIcechunk);
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
