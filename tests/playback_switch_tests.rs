//! Playback animates the plotted view while another variable is only selected
//! or a new plot (selection, plot type) is loading, and the new plot replaces
//! it, at its requested step, once its block arrives. Drives
//! `advance_playback` (the frame timer's step) headlessly against the offline
//! procedural 4D store.

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

/// A plotted 4D volume played to step 3, with every step resident.
fn playing(plot: PlotType) -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected_store_kind = StoreKind::ProceduralVolume4D;
    app.store_target_input = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.active_plot_type = plot;
    app.plot_selection();
    drain(&mut app);
    app.is_playing = true;
    for _ in 0..20 {
        if app.current_timestep == 3 {
            break;
        }
        tick(&mut app);
    }
    assert_eq!(app.current_timestep, 3, "playback reaches step 3");
    app
}

/// One timer step; a step whose block is not resident yet holds and prefetches.
fn tick(app: &mut OctantApp) {
    app.advance_playback(Instant::now());
    drain(app);
}

/// A frame: the timer step, then one poll (blocks may still be in flight).
fn frame(app: &mut OctantApp) {
    app.advance_playback(Instant::now());
    app.poll_block_prefetch_results();
    std::thread::sleep(Duration::from_millis(1));
}

/// The volume equals the analytic field at `t` on the box `lo..=hi` (all axes).
fn assert_volume(app: &OctantApp, t: usize, (lo, hi): (usize, usize), label: &str) {
    let v = app.volume_data.as_ref().expect("volume data");
    let n = hi - lo + 1;
    assert_eq!((v.width, v.height, v.depth), (n, n, n), "{label}: dims");
    for (i, got) in v.values.iter().enumerate() {
        let (x, y, z) = (i % n, (i / n) % n, i / (n * n));
        let expected =
            octant::data::eval_known_truth_4d(t, NT, lo + z, N, lo + y, N, lo + x, N, None);
        assert!(
            (got - expected).abs() < 1e-5,
            "{label}: voxel {i} = {got}, expected {expected}"
        );
    }
}

#[test]
fn a_new_plot_replaces_playback_at_its_step() {
    let mut app = playing(PlotType::Volume);
    // A smaller box: a new block, requested for the current step.
    for dim in 1..4 {
        app.selected_dim_ranges[dim] = (4, 19);
        app.dim_config[dim].range = (4, 19);
    }
    app.plot_selection();
    let step = app.current_timestep;
    // The plotted box keeps playing while the new one loads (no polls yet).
    app.advance_playback(Instant::now());
    assert_eq!(app.current_timestep, step + 1, "playback continues");
    assert_volume(&app, step + 1, (0, 31), "old box while loading");
    drain(&mut app);
    assert_eq!(app.current_timestep, step, "the new plot shows its step");
    assert_volume(&app, step, (4, 19), "new box");
    tick(&mut app);
    assert_eq!(app.current_timestep, step + 1, "and plays on");
    assert_volume(&app, step + 1, (4, 19), "new box playing");
}

#[test]
fn playback_continues_while_another_variable_is_selected() {
    let mut app = playing(PlotType::Volume);
    let plotted = app.plotted_variable_idx;
    let other = 1 - plotted;
    // What picking a row in the variables overlay does, without pressing Plot.
    app.selected_variable_idx = other;
    let info = app
        .active_dataset_metadata
        .as_ref()
        .expect("metadata")
        .variables[other]
        .clone();
    octant::ui::variables_panel::init_variable_dimension_defaults(&mut app, &info);
    for _ in 0..3 {
        tick(&mut app);
    }
    assert_eq!(app.current_timestep, 6, "playback continues");
    assert_eq!(
        app.plotted_variable_idx, plotted,
        "the plotted variable stays"
    );
    assert_eq!(
        app.selected_variable_idx, other,
        "the selection stays staged"
    );
    assert_volume(&app, 6, (0, 31), "plotted variable playing");
}

#[test]
fn switching_plot_type_while_playing_shows_the_new_layout() {
    // A heatmap of depth 5, playing with lookahead blocks in flight.
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected_store_kind = StoreKind::ProceduralVolume4D;
    app.store_target_input = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.active_plot_type = PlotType::Heatmap;
    app.selected_dim_ranges[1] = (5, 5);
    app.dim_config[1].range = (5, 5);
    app.selected_dim_indices[1] = 5;
    app.plot_selection();
    drain(&mut app);
    app.is_playing = true;
    for _ in 0..3 {
        frame(&mut app);
    }
    // The plot type menu with the full depth: what `plot_type.rs` does.
    app.active_plot_type = PlotType::Volume;
    app.selected_dim_ranges[1] = (0, 31);
    app.dim_config[1].range = (0, 31);
    app.load_selected_variable_block();
    for _ in 0..20 {
        frame(&mut app);
    }
    drain(&mut app);
    assert_eq!(app.plotted_plot_type, PlotType::Volume);
    assert_volume(
        &app,
        app.current_timestep,
        (0, 31),
        "volume after the switch",
    );
}
