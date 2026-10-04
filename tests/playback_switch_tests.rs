//! Playback must never show a selection other than the plotted one: it holds
//! while a new plot's block is loading and while another variable is only
//! selected. Drives `advance_playback` (the frame timer's step) headlessly
//! against the offline procedural 4D store.

use std::time::{Duration, Instant};

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

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
fn playing_volume() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected_store_kind = StoreKind::ProceduralVolume4D;
    app.store_target_input = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.active_plot_type = PlotType::Volume;
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

#[test]
fn playback_holds_while_a_new_plot_loads() {
    let mut app = playing_volume();
    // A smaller box: a new block, requested for the current step.
    for dim in 1..4 {
        app.selected_dim_ranges[dim] = (4, 19);
        app.dim_config[dim].range = (4, 19);
    }
    app.plot_selection();
    // Plot moves the step into the selected time range.
    let step = app.current_timestep;
    for _ in 0..5 {
        app.advance_playback(Instant::now());
        assert_eq!(
            app.current_timestep, step,
            "the step waits for the requested block"
        );
    }
    drain(&mut app);
    let v = app.volume_data.as_ref().expect("volume data");
    assert_eq!(
        (v.width, v.height, v.depth),
        (16, 16, 16),
        "the new box is shown"
    );
    for _ in 0..20 {
        if app.current_timestep != step {
            break;
        }
        tick(&mut app);
    }
    assert_eq!(
        app.current_timestep,
        step + 1,
        "playback resumes once it is shown"
    );
}

#[test]
fn playback_never_plots_a_merely_selected_variable() {
    let mut app = playing_volume();
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
        app.advance_playback(Instant::now());
        drain(&mut app);
    }
    assert_eq!(
        app.plotted_variable_idx, plotted,
        "the plotted variable stays"
    );
    assert_eq!(app.current_timestep, 3, "playback holds on its frame");
    assert_eq!(app.active_plot_type, PlotType::Volume);
}
