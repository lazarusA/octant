//! Detail tiles over an in-memory GeoZarr pyramid plotted at its coarsest
//! level: zooming in reads the visible part finer, zooming out, playing, a
//! pinned level or another step drop it.

use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::app::test_support::{poll_until, pyramid_app, selection_of};
use crate::plots::PlotType;

/// `t2m` plotted on a 20 px canvas: level 2 (16 x 16 of the 64 x 64 level 0).
fn plotted() -> OctantApp {
    let (mut app, meta) = pyramid_app();
    app.layout.canvas_px = [20, 20];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    app
}

fn view(zoom: f32) -> CanvasView {
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(20.0, 20.0));
    CanvasView {
        canvas_rect: rect,
        plot_rect: rect,
        pan: [0.0, 0.0],
        zoom,
        aspect_scale: [1.0, 1.0],
    }
}

fn tile_level(app: &OctantApp) -> Option<usize> {
    app.layers.base.data.detail.tile.as_ref().map(|t| t.level)
}

/// Refreshes at `zoom` and waits for the tile's block when one is asked.
fn rest_at(app: &mut OctantApp, zoom: f32) {
    app.refresh_detail(PlotType::Heatmap, &view(zoom));
    if app.layers.base.data.detail.pending.is_some() {
        poll_until(app, |app| app.layers.base.data.detail.pending.is_none());
    }
}

#[test]
fn zooming_in_reads_the_visible_window_finer() {
    let mut app = plotted();
    rest_at(&mut app, 1.0);
    assert_eq!(tile_level(&app), None, "level 2 fits the whole view");

    // Four times in: 4 level-2 cells across 20 px; level 0 has 16.
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    let tile = app.layers.base.data.detail.tile.clone().expect("tile");
    assert!(tile.cover[0].0 <= 24.0 && tile.cover[0].1 >= 40.0);
    assert!(tile.uv[0] <= 0.375 && tile.uv[2] >= 0.625);

    // Twice in: level 1 (16 cells over the visible 32 level-0 cells).
    rest_at(&mut app, 2.0);
    assert_eq!(tile_level(&app), Some(1));

    rest_at(&mut app, 1.0);
    assert_eq!(tile_level(&app), None, "zoomed out: the plot fits");
}

#[test]
fn moving_views_and_other_plots_ask_for_nothing() {
    let mut app = plotted();
    app.nav.view_interacting = true;
    app.refresh_detail(PlotType::Heatmap, &view(4.0));
    assert!(
        app.layers.base.data.detail.pending.is_none(),
        "not while zooming"
    );
    app.nav.view_interacting = false;

    app.playback.is_playing = true;
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), None, "not while playing");
    app.playback.is_playing = false;

    app.refresh_detail(PlotType::Surface, &view(4.0));
    assert!(
        app.layers.base.data.detail.pending.is_none(),
        "heatmaps only"
    );
}

#[test]
fn another_step_or_a_pinned_level_drops_the_tile() {
    let mut app = plotted();
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    app.playback.current_timestep = 1;
    app.refresh_detail(PlotType::Heatmap, &view(4.0));
    let tile = app.layers.base.data.detail.tile.as_ref();
    assert!(tile.is_none(), "the old step's tile is gone");
    poll_until(&mut app, |app| {
        app.layers.base.data.detail.pending.is_none()
    });
    assert_eq!(tile_level(&app), Some(0), "the new step's arrives");

    app.layers.base.selection_mut().level = LevelChoice::Pinned(2);
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), None, "a pinned level is shown as chosen");
}
