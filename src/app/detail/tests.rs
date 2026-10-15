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

thread_local! {
    /// The egui time of this test thread's frames: only moves on.
    static CLOCK: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

/// One frame at `zoom`, `seconds` after the last.
fn frame_at(app: &mut OctantApp, plot_type: PlotType, zoom: f32, seconds: f64) -> Option<f64> {
    let now = CLOCK.with(|c| {
        c.set(c.get() + seconds);
        c.get()
    });
    app.refresh_detail(plot_type, &view(zoom), now)
}

/// Rests at `zoom` (two frames past the settle time) and waits for the
/// tile's block when one is asked.
fn rest_at(app: &mut OctantApp, zoom: f32) {
    frame_at(app, PlotType::Heatmap, zoom, 1.0);
    frame_at(app, PlotType::Heatmap, zoom, 1.0);
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
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    assert!(
        app.layers.base.data.detail.pending.is_none(),
        "not while zooming"
    );
    app.nav.view_interacting = false;

    app.playback.is_playing = true;
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), None, "not while playing");
    app.playback.is_playing = false;

    frame_at(&mut app, PlotType::Surface, 4.0, 1.0);
    frame_at(&mut app, PlotType::Surface, 4.0, 1.0);
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
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    let tile = app.layers.base.data.detail.tile.as_ref();
    assert!(tile.is_none(), "the old step's tile is gone");
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0), "the new step's arrives");

    app.layers.base.selection_mut().level = LevelChoice::Pinned(2);
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), None, "a pinned level is shown as chosen");
}

#[test]
fn wheel_ticks_ask_for_nothing_until_the_view_rests() {
    let mut app = plotted();
    // Ticks a few frames apart, each zooming further in.
    for (i, zoom) in [2.0, 2.5, 3.0, 3.5, 4.0].into_iter().enumerate() {
        let wait = frame_at(&mut app, PlotType::Heatmap, zoom, 0.05);
        assert!(wait.is_some_and(|w| w > 0.0), "tick {i} waits");
        assert!(app.layers.base.data.detail.pending.is_none(), "tick {i}");
        assert_eq!(tile_level(&app), None);
    }
    let wait = frame_at(&mut app, PlotType::Heatmap, 4.0, 0.05);
    assert!(
        wait.is_some_and(|w| (w - 0.10).abs() < 1e-9),
        "the rest of the settle time"
    );
    assert!(frame_at(&mut app, PlotType::Heatmap, 4.0, 0.2).is_none());
    poll_until(&mut app, |app| {
        app.layers.base.data.detail.pending.is_none()
    });
    assert_eq!(tile_level(&app), Some(0), "only the resting view's tile");
}

#[test]
fn the_hover_reads_the_tile_where_it_is_drawn() {
    let mut app = plotted();
    let coarse_under = |app: &OctantApp| {
        let m = app.layers.base.data.matrix.as_ref().expect("matrix");
        m.values[6 * m.width + 6]
    };
    assert!(crate::ui::hover::detail::detail_cell(&app, 0.5, 0.5).is_none());

    rest_at(&mut app, 4.0);
    // At 4x the view shows level-0 cells 24..40 along each axis.
    let cell = crate::ui::hover::detail::detail_cell(&app, 0.4, 0.4).expect("over the tile");
    // Level 0 holds 0, 1, 2, ... row by row; the plot may show `lat` north-up.
    let flipped = app.layers.base.data.flipped_dims.iter().any(|d| d == "lat");
    let row = if flipped { 63 - 25 } else { 25 };
    assert_eq!((cell.x.1, cell.y.1), (25, row));
    assert_eq!(cell.value, (row * 64 + 25) as f32);
    assert_ne!(cell.value, coarse_under(&app), "not the coarse cell");

    let meta = app.plotted().metadata.clone();
    let var = app.plotted().variable_info().cloned();
    let matrix = app.layers.base.data.matrix.clone().expect("matrix");
    let (val, fields, _, _) = crate::ui::hover::entries_2d::resolve_2d_plot_entries(
        &app,
        &matrix,
        meta.as_ref(),
        var.as_ref(),
        0.4,
        0.4,
        None,
    );
    assert_eq!(val, cell.value);
    let shown: Vec<String> = fields.iter().map(|f| format!("{f:?}")).collect();
    // Shown counted from 1 out of level 0's 64.
    let (row, col) = (format!("{} / 64", row + 1), "26 / 64");
    let has = |text: &str| shown.iter().any(|f| f.contains(text));
    assert!(has(&row) && has(col), "level-0 indices: {shown:?}");
}
