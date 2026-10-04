//! Playback of single-step plots (heatmap, surface, sphere) through the real
//! loading path, headless, against the offline procedural 4D store: every
//! played step must show that step's slice.

use std::time::{Duration, Instant};

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

const NT: usize = 20;
const N: usize = 32;

fn drain(app: &mut OctantApp) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        app.poll_block_prefetch_results();
        if app.block_prefetcher.pending_count() == 0 {
            break;
        }
        assert!(Instant::now() < deadline, "prefetcher did not finish");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn app_with(plot: PlotType) -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected_store_kind = StoreKind::ProceduralVolume4D;
    app.store_target_input = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.active_plot_type = plot;
    app
}

/// The 2D slice shown for step `t` (dims: time, depth, lat, lon) at depth `z`.
fn assert_slice(app: &OctantApp, t: usize, z: usize, label: &str) {
    let m = app.matrix_data.as_ref().expect("matrix data");
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
    assert_eq!(app.animated_dim, Some(0));
    let z = app.selected_dim_indices[1];
    app.plot_selection();
    drain(&mut app);
    assert_slice(&app, 0, z, "first plot");
    for t in 1..NT {
        app.current_timestep = t;
        app.load_selected_variable_block();
        drain(&mut app);
        assert_eq!(app.current_timestep, t);
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
