//! Prefetch through the real loading path, headless, against the offline
//! procedural 4D store (20 time steps, one per chunk, a time axis without a
//! spatial role): a paused selected range loads whole within the cache,
//! playback keeps chunks ahead, and foreground requests leave the lookahead
//! its workers.

use crate::app::layers::LayerId;
use crate::app::{OctantApp, SpatialRole, StoreKind};
use crate::data::BlockStore;
use crate::data::backends::ProceduralBlockStore;
use crate::plots::PlotType;

const NT: usize = 20;
/// Bytes of one heatmap block (one step: the 32^3 depth, lat, lon slab).
const BLOCK_BYTES: usize = 32 * 32 * 32 * 4;

fn heatmap_app() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.plot_type = PlotType::Heatmap;
    assert_eq!(app.selected.animated_dim, Some(0));
    assert_eq!(app.selected.dim_config[0].spatial, SpatialRole::None);
    app
}

/// Selects steps `start..=end` of the time axis.
fn select_time(app: &mut OctantApp, (start, end): (usize, usize)) {
    app.selected.dim_ranges[0] = (start, end);
    app.selected.dim_config[0].range = (start, end);
}

/// Polls until no fetch is pending (each arrival queues more).
fn drain(app: &mut OctantApp) {
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        app.poll_block_prefetch_results();
        if app.block_prefetcher.pending_count() == 0 {
            return;
        }
        assert!(
            web_time::Instant::now() < deadline,
            "prefetch never settled"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn resident(app: &OctantApp) -> Vec<usize> {
    (0..NT)
        .filter(|&t| app.layer_step_resident(LayerId::BASE, t))
        .collect()
}

#[test]
fn a_paused_time_range_loads_every_selected_step() {
    let mut app = heatmap_app();
    select_time(&mut app, (2, 13));
    app.playback.current_timestep = 2;
    app.plot_selection();
    drain(&mut app);
    assert_eq!(resident(&app), (2..=13).collect::<Vec<_>>());
}

#[test]
fn a_range_larger_than_the_cache_keeps_the_shown_step() {
    let mut app = heatmap_app();
    // Four blocks: the range may hold three (three quarters of the cache).
    app.block_cache.set_max_bytes(4 * BLOCK_BYTES + 1024);
    select_time(&mut app, (0, NT - 1));
    app.plot_selection();
    drain(&mut app);
    assert_eq!(resident(&app), vec![0, 1, 2]);
}

#[test]
fn playback_keeps_half_the_cache_ahead() {
    let mut app = heatmap_app();
    // Eight blocks: four chunks ahead (half the cache), not two.
    app.block_cache.set_max_bytes(8 * BLOCK_BYTES + 1024);
    app.plot_selection();
    drain(&mut app);
    app.playback.is_playing = true;
    for _ in 0..6 {
        app.advance_playback(web_time::Instant::now());
        drain(&mut app);
        let t = app.playback.current_timestep;
        for ahead in t + 1..=t + 3 {
            assert!(
                app.layer_step_resident(LayerId::BASE, ahead % NT),
                "step {ahead} is resident while playing step {t}"
            );
        }
    }
}

#[test]
fn a_pending_foreground_block_leaves_the_lookahead_a_worker() {
    let mut app = heatmap_app();
    app.prefetch_threads = 1;
    app.plot_selection();
    drain(&mut app);
    // Step 3 is not resident: its block is the shown (foreground) request.
    app.playback.current_timestep = 3;
    app.load_selected_variable_block();
    assert_eq!(app.block_prefetcher.pending_count(), 1, "the shown block");
    app.prefetch_animated_ranges();
    assert_eq!(
        app.block_prefetcher.pending_count(),
        2,
        "one lookahead chunk beside the shown block"
    );
}
