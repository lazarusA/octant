//! How far a heatmap zooms in over a coarse level (and back within a lower
//! limit), and a failed tile fetch asked again once while the view rests.

use super::tests::{frame_at, plotted, tile_level};
use super::want::max_zoom_over;
use crate::app::controllers::controller_for;
use crate::app::layers::LevelChoice;
use crate::app::state::navigation::MAX_ZOOM;
use crate::plots::PlotType;

#[test]
fn coarse_levels_zoom_until_level_0_cells_span_pixels() {
    assert_eq!(max_zoom_over(1.0), MAX_ZOOM);
    assert_eq!(max_zoom_over(4.0), MAX_ZOOM, "never below the default");
    // GLC_FCS30D's 1.3 million columns plotted from a level 1024 times coarser.
    assert_eq!(max_zoom_over(1024.0), 4096.0);
    assert_eq!(max_zoom_over(1e6), 1e4, "within f32's reach");
}

#[test]
fn the_plotted_level_sets_the_heatmap_zoom_limit() {
    let mut app = plotted();
    // Level 2 is 4 times coarser: 16 pixels per level-0 cell is under 50.
    assert_eq!(app.heatmap_max_zoom(), MAX_ZOOM);
    app.layers.base.selection_mut().level = LevelChoice::Pinned(2);
    assert_eq!(app.heatmap_max_zoom(), MAX_ZOOM, "pinned: no finer tiles");
}

#[test]
fn heatmaps_zoom_to_the_limit_their_plot_sets() {
    let mut nav = crate::app::state::NavigationState::default();
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0));
    let heatmap = controller_for(PlotType::Heatmap);
    nav.heatmap_max_zoom = 4096.0;
    for _ in 0..40 {
        heatmap.handle_zoom(&mut nav, 2.0, rect.center(), rect);
    }
    assert_eq!(nav.heatmap_zoom, 4096.0);
    let line = controller_for(PlotType::Line);
    for _ in 0..40 {
        line.handle_zoom(&mut nav, 2.0, rect.center(), rect);
    }
    assert_eq!(nav.line_zoom, MAX_ZOOM, "line plots keep the default");
}

#[test]
fn a_failed_tile_is_asked_again_while_the_view_rests() {
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
    let key = key.expect("tile fetch");
    assert!(app.take_detail_result(&key, &Err("connection reset".into())));
    // The view never moves; the tile still arrives.
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while tile_level(&app).is_none() {
        assert!(web_time::Instant::now() < deadline, "never asked again");
        frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
        app.poll_block_prefetch_results();
        std::thread::yield_now();
    }
    assert_eq!(tile_level(&app), Some(0));
}

#[test]
fn a_tile_that_keeps_failing_is_asked_again_once() {
    let mut state = super::DetailState::default();
    let key = super::ViewKey {
        pan: [0.0; 2],
        zoom: 4.0,
        aspect: [1.0; 2],
        px: [20, 20],
        context: 1,
    };
    state.view = Some(key);
    state.retry_failed();
    assert_eq!(state.view, None, "asked again");
    state.view = Some(key);
    state.retry_failed();
    assert_eq!(state.view, Some(key), "not a second time");
}

#[test]
fn a_lower_limit_zooms_out_about_the_center() {
    let mut nav = crate::app::state::NavigationState {
        heatmap_zoom: 2000.0,
        heatmap_pan: egui::vec2(400.0, -80.0),
        ..Default::default()
    };
    nav.limit_heatmap_zoom(MAX_ZOOM);
    assert_eq!(nav.heatmap_zoom, MAX_ZOOM);
    assert_eq!(
        nav.heatmap_pan,
        egui::vec2(10.0, -2.0),
        "the same point centered"
    );
    nav.limit_heatmap_zoom(4096.0);
    assert_eq!(nav.heatmap_zoom, MAX_ZOOM, "a higher limit leaves the view");
}
