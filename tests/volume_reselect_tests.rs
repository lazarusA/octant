//! Re-plotting a volume with a different slider selection must fill the new
//! volume: drives the real block loading path (`load_selected_variable_block`,
//! the async prefetcher and `poll_block_prefetch_results`) headlessly against
//! the offline procedural 4D store.

use std::time::{Duration, Instant};

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

const NT: usize = 20;
const N: usize = 32;

fn new_volume_app() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.selected.plot_type = PlotType::Volume;
    app
}

/// What the Plot button does (`src/ui/variables_panel/mod.rs`), then drains
/// the prefetcher the way the frame loop does.
fn press_plot(app: &mut OctantApp) {
    app.plot_selection();
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

/// Sets the slider range of `dim` the way `slider_row.rs` does.
fn set_range(app: &mut OctantApp, dim: usize, range: (usize, usize)) {
    app.selected.dim_ranges[dim] = range;
    app.selected.dim_config[dim].range = range;
}

/// Checks the volume against the analytic field for `ranges` (z, y, x) at `t`.
fn assert_volume_matches(app: &OctantApp, t: usize, ranges: [(usize, usize); 3], label: &str) {
    let vdata = app.layers.base.data.volume.as_ref().expect("volume data");
    let [(z0, z1), (y0, y1), (x0, x1)] = ranges;
    let (nz, ny, nx) = (z1 - z0 + 1, y1 - y0 + 1, x1 - x0 + 1);
    assert_eq!(
        (vdata.width, vdata.height, vdata.depth),
        (nx, ny, nz),
        "{label}: volume dims"
    );
    let nan = vdata.values.iter().filter(|v| v.is_nan()).count();
    assert_eq!(
        nan,
        0,
        "{label}: {nan} of {} voxels are NaN",
        vdata.values.len()
    );
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let expected =
                    octant::data::eval_known_truth_4d(t, NT, z0 + z, N, y0 + y, N, x0 + x, N, None);
                let got = vdata.values[z * nx * ny + y * nx + x];
                assert!(
                    (got - expected).abs() < 1e-5,
                    "{label}: voxel ({x},{y},{z}) = {got}, expected {expected}"
                );
            }
        }
    }
    assert!(
        app.layers.base.color.volume_cmin.is_finite()
            && app.layers.base.color.volume_cmax.is_finite(),
        "{label}: color range not finite"
    );
    assert!(
        app.layers.base.color.volume_cmin >= vdata.min_val - 1e-5
            && app.layers.base.color.volume_cmax <= vdata.max_val + 1e-5,
        "{label}: color range {}..{} outside data {}..{}",
        app.layers.base.color.volume_cmin,
        app.layers.base.color.volume_cmax,
        vdata.min_val,
        vdata.max_val
    );
}

#[test]
fn volume_replot_with_smaller_then_larger_spatial_ranges() {
    let mut app = new_volume_app();
    // dims: time(0, animated), depth(1, Z), lat(2, Y), lon(3, X)
    assert_eq!(app.selected.animated_dim, Some(0));

    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "first plot");

    set_range(&mut app, 1, (4, 20));
    set_range(&mut app, 2, (8, 23));
    set_range(&mut app, 3, (2, 29));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(4, 20), (8, 23), (2, 29)], "smaller plot");

    set_range(&mut app, 1, (0, 31));
    set_range(&mut app, 2, (0, 31));
    set_range(&mut app, 3, (0, 31));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "larger plot");
}

#[test]
fn volume_replot_after_changing_animated_range() {
    let mut app = new_volume_app();
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "first plot");

    // Wider time range plus a smaller spatial box: the extra time chunks are
    // prefetched in the background and must not overwrite timestep 0.
    set_range(&mut app, 0, (0, 5));
    set_range(&mut app, 1, (4, 20));
    set_range(&mut app, 2, (8, 23));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(4, 20), (8, 23), (0, 31)], "time range plot");

    // Back to a bigger box with the same time range.
    set_range(&mut app, 1, (0, 31));
    set_range(&mut app, 2, (0, 31));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "bigger after time");
}

/// Animates the depth axis (Z) with `chunk`-deep blocks: the volume is filled
/// by several blocks along Z (the 3D spatial-animation ring buffer path).
fn new_depth_animated_app(chunk: u64) -> OctantApp {
    use octant::app::{AnimationRole, DimConfig, SpatialRole};
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let mut meta = store.inspect().expect("inspect procedural store");
    meta.variables[0].chunk_shape = vec![1, chunk, 32, 32];
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.selected.plot_type = PlotType::Volume;
    let role = |spatial, animation, range| DimConfig {
        spatial,
        animation,
        active: true,
        index: 0,
        range,
    };
    app.selected.dim_config = vec![
        DimConfig::default(),
        role(SpatialRole::Z, AnimationRole::Animated, (0, 31)),
        role(SpatialRole::Y, AnimationRole::None, (0, 31)),
        role(SpatialRole::X, AnimationRole::None, (0, 31)),
    ];
    app.selected.dim_config[0].active = false;
    app.selected.animated_dim = Some(1);
    app.selected.spatial_dims = vec![3, 2, 1];
    app.selected.dim_indices = vec![0; 4];
    app.selected.dim_ranges = vec![(0, 0), (0, 31), (0, 31), (0, 31)];
    app.current_timestep = 0;
    app
}

#[test]
fn depth_animated_volume_replot_smaller_then_larger() {
    let mut app = new_depth_animated_app(8);

    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "first plot");

    set_range(&mut app, 1, (4, 20));
    set_range(&mut app, 2, (8, 23));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(4, 20), (8, 23), (0, 31)], "smaller plot");

    set_range(&mut app, 1, (0, 31));
    set_range(&mut app, 2, (0, 31));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "larger plot");
}

#[test]
fn depth_animated_volume_replot_changing_only_the_animated_range() {
    let mut app = new_depth_animated_app(8);

    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "first plot");

    // Only the animated Z range changes: the cached Z blocks still match.
    set_range(&mut app, 1, (4, 20));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(4, 20), (0, 31), (0, 31)], "smaller Z");

    // A Z range that excludes the current step's block.
    set_range(&mut app, 1, (16, 31));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(16, 31), (0, 31), (0, 31)], "Z past the step");

    set_range(&mut app, 1, (0, 31));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "larger Z");
}

#[test]
fn depth_animated_volume_plot_with_step_past_the_range() {
    // The step stays where playback left it, past the newly chosen Z range.
    let mut app = new_depth_animated_app(8);
    app.current_timestep = 20;
    set_range(&mut app, 1, (0, 15));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 15), (0, 31), (0, 31)], "step past range");
}

/// What the playback timer does each frame (`src/app/ui.rs`): advance the step
/// and load, then drain the prefetcher.
fn play_to(app: &mut OctantApp, t: usize) {
    app.current_timestep = t;
    app.load_selected_variable_block();
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

#[test]
fn volume_playback_steps_past_the_selected_time_window() {
    let mut app = new_volume_app();
    // A short time window, as the memory budget picks for large datasets:
    // playback still runs over every timestep.
    set_range(&mut app, 0, (0, 3));
    press_plot(&mut app);
    assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "first plot");
    for t in 1..8 {
        play_to(&mut app, t);
        assert_eq!(app.current_timestep, t, "playback must reach step {t}");
        assert_volume_matches(&app, t, [(0, 31), (0, 31), (0, 31)], "playback");
    }
}

#[test]
fn depth_animated_volume_playback_keeps_the_full_volume() {
    // The depth axis is animated: the volume is a ring buffer of its Z blocks,
    // and playback moves through it without losing any of them.
    let mut app = new_depth_animated_app(8);
    press_plot(&mut app);
    for t in 1..12 {
        play_to(&mut app, t);
        assert_eq!(app.current_timestep, t, "playback must reach step {t}");
        assert_volume_matches(&app, 0, [(0, 31), (0, 31), (0, 31)], "depth playback");
    }
}
