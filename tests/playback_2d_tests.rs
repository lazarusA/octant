//! Playback of single-step plots (heatmap, surface, sphere) through the real
//! loading path, headless, against the offline procedural 4D store: every
//! played step must show that step's slice.

mod common;
use common::drain;
use std::time::{Duration, Instant};

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

const NT: usize = 20;
const N: usize = 32;

fn app_with(plot: PlotType) -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.plot_type = plot;
    app
}

/// The 2D slice shown for step `t` (dims: time, depth, lat, lon) at depth `z`.
fn assert_slice(app: &OctantApp, t: usize, z: usize, label: &str) {
    let m = app.layers.base.data.matrix.as_ref().expect("matrix data");
    assert_eq!((m.width, m.height), (N, N), "{label}: slice dims");
    for y in 0..N {
        for x in 0..N {
            let expected = octant::data::eval_known_truth_4d(t, NT, z, N, y, N, x, N, None);
            let got = m.values[y * N + x];
            assert!(
                (got - expected).abs() < 1e-5,
                "{label} step {t}: ({x},{y}) = {got}, expected {expected}"
            );
        }
    }
}

fn play(plot: PlotType) {
    let mut app = app_with(plot);
    assert_eq!(app.selected.animated_dim, Some(0));
    let z = app.selected.dim_indices[1];
    app.plot_selection();
    drain(&mut app);
    assert_slice(&app, 0, z, "first plot");
    for t in 1..NT {
        app.playback.current_timestep = t;
        app.load_selected_variable_block();
        drain(&mut app);
        assert_eq!(app.playback.current_timestep, t);
        assert_slice(&app, t, z, "playback");
    }
}

#[test]
fn heatmap_playback_shows_every_step() {
    play(PlotType::Heatmap);
}

#[test]
fn surface_playback_shows_every_step() {
    play(PlotType::Surface);
}

#[test]
fn sphere_playback_shows_every_step() {
    play(PlotType::Sphere);
}

/// Playback through the frame timer's step (`advance_playback`) runs past the
/// selected time window as the prefetcher delivers the next slices.
fn play_past_window(plot: PlotType, cache_blocks: usize) {
    let mut app = app_with(plot);
    // Room for `cache_blocks` per-step slabs (each 32^3 f32 values).
    app.block_cache
        .set_max_bytes(cache_blocks * N * N * N * 4 + 1024);
    let z = app.selected.dim_indices[1];
    app.selected.dim_ranges[0] = (0, 3);
    app.selected.dim_config[0].range = (0, 3);
    app.plot_selection();
    drain(&mut app);
    app.playback.is_playing = true;
    // Frames: a timer step, then polls for a few milliseconds.
    for _ in 0..400 {
        if app.playback.current_timestep >= 8 {
            break;
        }
        app.advance_playback(Instant::now());
        // What the open variables panel writes every frame (`slider_row.rs`).
        app.selected.dim_indices[0] = app.playback.current_timestep.clamp(0, 3);
        app.selected.dim_config[0].index = app.selected.dim_indices[0];
        for _ in 0..5 {
            app.poll_block_prefetch_results();
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(
        app.playback.current_timestep, 8,
        "playback must run past the window"
    );
    assert_slice(&app, 8, z, "past the window");
}

#[test]
fn heatmap_playback_runs_past_the_selected_window() {
    play_past_window(PlotType::Heatmap, 64);
}

#[test]
fn heatmap_playback_past_the_window_with_a_small_cache() {
    play_past_window(PlotType::Heatmap, 4);
}
