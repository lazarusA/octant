//! Detail tile fetches: refused while every worker is busy, shared with a
//! layer's own block, and kept over a small pan.

use super::tests::{frame_at, frame_with, plotted, rest_at, tile_level, view};
use crate::app::OctantApp;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::app::test_support::poll_until;
use crate::plots::PlotType;

/// Another block of the store, fetched in the background.
fn other_block(app: &OctantApp) -> crate::data::BlockRequest {
    let sel = app.plotted().clone();
    let source_id = app.selection_source_id(&sel);
    let handle = app
        .resolve_store_handle(&source_id, &sel.store_target, sel.store_kind)
        .expect("store");
    let slice = crate::data::SliceRequest::full_range("2/sst", &[3, 16, 16]);
    crate::data::BlockRequest::new(handle, slice)
}

#[test]
fn a_tile_refused_by_busy_workers_is_asked_again_once_one_frees() {
    let mut app = plotted();
    app.block_prefetcher = crate::data::BlockPrefetcher::with_max_concurrent_threads(1);
    let busy = other_block(&app);
    assert!(app.block_prefetcher.request(busy, &app.block_cache));
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    let state = &app.layers.base.data.detail;
    assert!(state.pending.is_none() && state.tile.is_none(), "refused");
    // The busy fetch completes; the tile is asked for again and arrives.
    poll_until(&mut app, |app| tile_level(app).is_some());
    assert_eq!(tile_level(&app), Some(0));
}

#[test]
fn a_layer_waiting_on_the_tile_fetch_gets_the_block_too() {
    let mut app = plotted();
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    let key = app
        .layers
        .base
        .data
        .detail
        .pending
        .as_ref()
        .map(|p| p.key.clone());
    // The base layer asks for the same block (one fetch serves both).
    app.layers.base.load.block_key = Some(key.expect("tile fetch"));
    poll_until(&mut app, |app| app.layers.base.load.block_key.is_none());
    assert_eq!(tile_level(&app), Some(0), "the tile is shown too");
}

#[test]
fn a_small_pan_keeps_the_tile() {
    let mut app = plotted();
    rest_at(&mut app, 4.0);
    let before = app.layers.base.data.detail.tile.clone().expect("tile");
    let panned = CanvasView {
        pan: [0.02, -0.02],
        ..view(4.0)
    };
    frame_with(&mut app, PlotType::Heatmap, &panned, 1.0);
    frame_with(&mut app, PlotType::Heatmap, &panned, 1.0);
    let state = &app.layers.base.data.detail;
    assert!(state.pending.is_none(), "nothing asked: the tile covers it");
    let after = state.tile.as_ref().expect("kept");
    assert_eq!((after.origin, after.end), (before.origin, before.end));
}
