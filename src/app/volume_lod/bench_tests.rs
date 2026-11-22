//! Benchmark of volume LOD on Beechnut (1546 x 1024 x 1024 uint16, 5
//! levels; Auto plots level 2), through the desktop loading path and a
//! headless GPU: time to the plotted volume and to the root bricks; for a
//! close-up from outside and a view from inside, the bricks read per level
//! and time to settle, CPU time per refresh and for the
//! visibility rays; GPU frame times of the dense level-2 volume, the
//! bricked one at rest (with and without empty skipping) and moving.
//! Run by hand with
//! `cargo test --release --lib volume_lod_beechnut -- --ignored --nocapture --test-threads 1`.

use std::time::{Duration, Instant};

use crate::app::pipeline::paint_layer::CanvasView;
use crate::app::{OctantApp, StoreKind};
use crate::data::{DataSource, Dataset, SourceFactory};
use crate::plots::PlotType;

#[path = "bench_feedback.rs"]
mod feedback;
#[path = "bench_frames.rs"]
mod frames;
use feedback::report_feedback;
use frames::{compare_images, report_frames, time_visibility};

const BEECHNUT: &str = "https://d39zu0xtgv0613.cloudfront.net/beechnut.ome.zarr";
/// Canvas of the benchmark, in pixels.
const PX: [u32; 2] = [1280, 960];
/// Frames timed per renderer.
const RUNS: usize = 9;

/// Beechnut open and plotted as a volume (Auto), with a GPU when there is
/// one; the time its plotted level took.
fn plotted() -> (OctantApp, Duration) {
    let mut app = OctantApp::default();
    app.wgpu_render_state = crate::app::test_support::render_state();
    let kind = StoreKind::RemoteZarr;
    let source_id = StoreKind::make_source_id(kind, BEECHNUT);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), BEECHNUT, "Store");
    let handle = SourceFactory::open(source.clone()).expect("open store");
    let meta = handle.inspect().expect("inspect store");
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_kind = kind;
    app.selected.store_target = BEECHNUT.to_string();
    app.load_new_metadata(meta.clone());
    app.layout.show_hero = false;
    app.layout.canvas_px = PX;
    let idx = meta
        .variables
        .iter()
        .position(|v| v.multiscale.is_some())
        .expect("a multiscale variable");
    let var = meta.variables[idx].clone();
    app.selected.variable_idx = idx;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    app.selected.plot_type = PlotType::Volume;
    let z = var.shape[0] as usize;
    app.selected.dim_config[0].range = (0, z - 1);
    let start = Instant::now();
    app.plot_selection();
    crate::app::test_support::poll_until_for(&mut app, Duration::from_secs(300), |a| {
        a.layers
            .base
            .data
            .volume
            .as_ref()
            .is_some_and(|v| v.min_val.is_finite())
    });
    // The dense volume uploads before it is painted: no paint here.
    app.flush_volume_uploads();
    (app, start.elapsed())
}

fn view() -> CanvasView {
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(PX[0] as f32, PX[1] as f32));
    CanvasView {
        canvas_rect: rect,
        plot_rect: rect,
        pan: [0.0, 0.0],
        zoom: 1.0,
        aspect_scale: [1.0, 1.0],
    }
}

/// What one settle took: wall time, the slowest refresh, bricks per level
/// resident and wanted.
struct Settle {
    wall: Duration,
    slowest: Duration,
    refreshes: usize,
    levels: Vec<usize>,
}

/// Frames 0.2 s apart (egui time) until the view rested and every wanted
/// brick was placed.
fn settle(app: &mut OctantApp, clock: &mut f64) -> Settle {
    let start = Instant::now();
    let (mut slowest, mut refreshes) = (Duration::ZERO, 0);
    loop {
        *clock += 0.2;
        let t = Instant::now();
        app.refresh_volume_lod(PlotType::Volume, &view(), *clock);
        slowest = slowest.max(t.elapsed());
        refreshes += 1;
        app.poll_block_prefetch_results();
        let lod = app
            .layers
            .base
            .data
            .volume_lod
            .lod
            .as_ref()
            .expect("bricked");
        let idle = lod.queue.is_empty() && lod.pending.is_empty() && lod.arrived.is_empty();
        if idle && refreshes > 3 {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(600), "never settled");
        std::thread::sleep(Duration::from_millis(5));
    }
    let lod = app
        .layers
        .base
        .data
        .volume_lod
        .lod
        .as_ref()
        .expect("bricked");
    let mut levels = vec![0; lod.set.len()];
    for k in lod.wanted.iter().filter(|k| lod.residency.holds(k)) {
        levels[usize::from(k.level)] += 1;
    }
    Settle {
        wall: start.elapsed(),
        slowest,
        refreshes,
        levels,
    }
}

#[test]
#[ignore = "downloads Beechnut and times the GPU: run by hand in release"]
fn volume_lod_beechnut_timed() {
    let (mut app, plotted) = plotted();
    let level = app
        .plotted()
        .variable_info()
        .and_then(|v| v.multiscale.as_ref())
        .map(|r| r.level);
    eprintln!("plotted level {level:?} in {:.1} s", plotted.as_secs_f64());
    app.plot_configs.volume.lod = true;
    app.plot_configs.volume.opacity = 3.0;
    // The range the nut was explored in: air below it draws nothing.
    app.layers.base.color.set_range(1.33e4, 3.37e4, true);
    let mut clock = 0.0;
    let roots = settle(&mut app, &mut clock);
    report(&roots, "roots");
    report_frames(&mut app, "roots");
    compare_images(&app, "roots");

    // Close in on the middle of the front face from just outside.
    app.nav.sphere_rotation_x = 0.0;
    app.nav.sphere_rotation_y = 0.0;
    app.nav.orbit_target = [0.0, 0.0, 0.0];
    app.nav.sphere_zoom = 0.75;
    let outside = settle(&mut app, &mut clock);
    report(&outside, "close-up outside");
    diagnose(&app, "close-up outside");
    report_frames(&mut app, "close-up outside");
    report_feedback(&mut app, &mut clock, "close-up outside");

    // From inside, near the center.
    app.nav.sphere_zoom = 0.15;
    let inside = settle(&mut app, &mut clock);
    report(&inside, "inside");
    diagnose(&app, "inside");
    report_frames(&mut app, "inside");
    report_feedback(&mut app, &mut clock, "inside");
    time_visibility(&app);
}

/// Why a view wants what it wants: bricks selected with no skipping, with
/// the empty test only, and with visibility too.
fn diagnose(app: &OctantApp, label: &str) {
    let lod = app
        .layers
        .base
        .data
        .volume_lod
        .lod
        .as_ref()
        .expect("bricked");
    let key = app.layers.base.data.volume_lod.view.expect("rested");
    let bricks = lod.bricks();
    let view = super::select::View {
        camera: &key.camera,
        place: lod.place,
        window: lod.window.clone(),
        px_height: key.px[1],
        threshold: 1.5 / key.quality,
    };
    let count = |empty: &dyn Fn(crate::data::multiscale::bricks::BrickKey) -> bool| {
        let none = |_| false;
        let skip = super::select::Skip {
            empty,
            hidden: &none,
            refined: &none,
        };
        super::select::select(&bricks, lod.root_level, &view, 10_000, skip).len()
    };
    eprintln!(
        "{label}: place {:?}, eye {:?}, distance {}, px {:?}, quality {}, root level {}, levels {}",
        lod.place,
        key.camera.eye(),
        key.camera.distance,
        key.px,
        key.quality,
        lod.root_level,
        lod.set.len()
    );
    let all = count(&|_| false);
    let nonempty = count(&|k| lod.draws_nothing(k, key.shown));
    eprintln!(
        "{label}: selected without skipping {all}, without empty bricks {nonempty}, wanted {}",
        lod.wanted.len()
    );
}

fn report(s: &Settle, label: &str) {
    eprintln!(
        "{label}: settled in {:.1} s, {} refreshes (slowest {:.1} ms), bricks per level {:?}",
        s.wall.as_secs_f64(),
        s.refreshes,
        s.slowest.as_secs_f64() * 1e3,
        s.levels,
    );
}
