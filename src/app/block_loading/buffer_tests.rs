//! Buffered playback through the real loading path, headless: against a
//! procedural store slowed down like a network, playback waits for its
//! buffer, never skips a step, and plays a whole buffer before stalling
//! again; a drawn overlay holds playback until it has the step too.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::test_support::{make_resident, memory_app, selection_of};
use crate::app::{OctantApp, StoreKind};
use crate::data::backends::ProceduralBlockStore;
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::{DataSource, Dataset, SliceRequest, StoreHandle};
use crate::plots::PlotType;

const NT: usize = 20;
const TARGET: &str = "procedural://volume4d";

/// The procedural store, each fetch taking `delay` like a slow network.
struct SlowStore {
    inner: ProceduralBlockStore,
    delay: Duration,
}

impl BlockStore for SlowStore {
    fn backend_name(&self) -> &str {
        "slow"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.inner.variables()
    }

    fn inspect(&self) -> Result<crate::data::DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        std::thread::sleep(self.delay);
        self.inner.fetch_block_with_progress(request, on_progress)
    }
}

/// A heatmap of the procedural 4D variable over a store whose fetches take
/// `delay`, its first step shown, one lookahead worker.
fn slow_app(delay: Duration) -> OctantApp {
    let inner = ProceduralBlockStore::open(TARGET).expect("open procedural store");
    let meta = inner.inspect().expect("inspect");
    let kind = StoreKind::ProceduralVolume4D;
    let source_id = StoreKind::make_source_id(kind, TARGET);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), TARGET, "Store");
    let handle = StoreHandle::new(source.clone(), Arc::new(SlowStore { inner, delay }));
    let mut app = OctantApp::default();
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_kind = kind;
    app.selected.store_target = TARGET.to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.plot_type = PlotType::Heatmap;
    app.prefetch_threads = 1;
    app.plot_selection();
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.layers.base.data.matrix.is_none() {
        assert!(Instant::now() < deadline, "the first block never arrived");
        app.poll_block_prefetch_results();
        std::thread::sleep(Duration::from_millis(1));
    }
    app
}

/// Plays to the last step as the frame timer does, recording each change of
/// the shown step or of buffering.
fn play_to_end(app: &mut OctantApp) -> Vec<(usize, bool)> {
    app.playback.loop_playback = false;
    app.toggle_playback();
    let frame = Duration::from_secs_f32(1.0 / app.playback.playback_fps);
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut log = vec![(app.playback.current_timestep, app.playback.buffering)];
    while app.playback.is_playing {
        assert!(Instant::now() < deadline, "playback never reached the end");
        let now = Instant::now();
        if now.duration_since(app.playback.last_step_time) >= frame {
            app.advance_playback(now);
        }
        app.poll_block_prefetch_results();
        let state = (app.playback.current_timestep, app.playback.buffering);
        if log.last() != Some(&state) {
            log.push(state);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    log
}

#[test]
fn playback_buffers_then_plays_a_whole_buffer_without_skipping() {
    // Blocks arrive about every 60 ms, frames every 20 ms: playback outruns
    // the store and has to buffer.
    let mut app = slow_app(Duration::from_millis(60));
    app.playback.playback_fps = 50.0;
    app.playback.buffer_secs = 0.2;
    let target = app.buffer_target_steps();
    assert_eq!(target, 10);
    let log = play_to_end(&mut app);

    let steps: Vec<usize> = log.iter().map(|&(s, _)| s).collect();
    assert!(
        steps.windows(2).all(|w| w[1] == w[0] || w[1] == w[0] + 1),
        "steps never skip: {log:?}"
    );
    assert_eq!(steps.last(), Some(&(NT - 1)), "plays to the end");

    // Steps played between buffering episodes: each whole run covers the buffer.
    let mut runs = vec![0usize];
    for w in log.windows(2) {
        let ((s0, _), (s1, buffering)) = (w[0], w[1]);
        if buffering && !w[0].1 {
            runs.push(0);
        }
        if s1 > s0 {
            *runs.last_mut().expect("a run") += 1;
        }
    }
    assert!(runs.len() > 1, "playback had to buffer: {log:?}");
    let whole = &runs[..runs.len() - 1];
    assert!(
        whole.iter().all(|&n| n >= target),
        "each resume plays at least {target} steps: {runs:?}"
    );
}

#[test]
fn a_fast_store_plays_without_buffering_once_started() {
    let mut app = slow_app(Duration::ZERO);
    app.prefetch_threads = 16;
    app.playback.playback_fps = 50.0;
    app.playback.buffer_secs = 0.2;
    let log = play_to_end(&mut app);
    let episodes = log.windows(2).filter(|w| w[1].1 && !w[0].1).count();
    assert_eq!(episodes, 0, "no stall after the start: {log:?}");
}

/// `t2m` plotted from resident blocks with `sst` overlaid, its blocks not
/// yet loaded; the overlay's id.
fn overlay_app() -> (OctantApp, crate::app::layers::LayerId) {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    let sst = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "sst")
        .expect("sst");
    let id = app.add_overlay(sst).expect("overlay");
    assert!(app.layers.get(id).is_some_and(|l| l.is_drawn()));
    app.playback.is_playing = true;
    (app, id)
}

#[test]
fn an_overlay_without_the_step_holds_playback() {
    let (mut app, _) = overlay_app();
    app.advance_playback(Instant::now());
    assert_eq!(app.playback.current_timestep, 0, "playback waits");
    assert!(app.playback.buffering);
    // Buffering wants the whole (looping) axis of 3 steps; none is ready.
    app.advance_playback(Instant::now());
    assert_eq!(app.playback.buffer_progress, (0, 3));

    make_resident(&mut app, &SliceRequest::full_range("sst", &[3, 5, 4]));
    app.advance_playback(Instant::now());
    assert_eq!(app.playback.current_timestep, 1, "both layers hold step 1");
    assert!(!app.playback.buffering);
}

#[test]
fn an_overlay_whose_fetch_failed_no_longer_holds_playback() {
    let (mut app, id) = overlay_app();
    if let Some(layer) = app.layers.get_mut(id) {
        layer.load.failed = true;
    }
    app.advance_playback(Instant::now());
    assert_eq!(app.playback.current_timestep, 1, "the base plays on");
}
