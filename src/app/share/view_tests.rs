//! The heatmap's zoom window through a link: written as level-0 cells,
//! shown again on the receiver's canvas once its plot draws.

use std::sync::Arc;

use super::parse_share;
use crate::app::OctantApp;
use crate::app::actions::{AppAction, Origin};
use crate::app::test_support::{poll_until, pyramid_app, selection_of};
use crate::data::ViewportResampler;
use crate::plots::PlotType;

const CANVAS: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(400.0, 400.0));

/// The pyramid's `0/t2m` plotted at its coarsest level (a 20 px canvas).
fn plotted_pyramid() -> OctantApp {
    let (mut app, meta) = pyramid_app();
    app.layout.canvas_px = [20, 20];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    app
}

/// What the heatmap shows on `CANVAS`, as `render_canvas` records it.
fn shown(app: &OctantApp) -> ((f64, f64), (f64, f64)) {
    let (w, h) = (CANVAS.width(), CANVAS.height());
    let pan = app.nav.heatmap_pan;
    let gpu = [pan.x / (0.5 * w), -pan.y / (0.5 * h)];
    let aspect = app.compute_aspect_scale(CANVAS.size());
    ViewportResampler::compute_visible_data_bounds(gpu, app.nav.heatmap_zoom, aspect)
}

#[test]
fn view_reads_into_set_view_after_the_colors() {
    let actions =
        parse_share("?src=https://x.org/a&var=t&view=10..20,-5..5&range=1..2").unwrap_or_default();
    let set_view = AppAction::SetView {
        x_from: "10".into(),
        x_to: "20".into(),
        y_from: "-5".into(),
        y_to: "5".into(),
    };
    assert_eq!(actions.last(), Some(&set_view));
    assert!(matches!(
        actions[actions.len() - 2],
        AppAction::SetColorRange { .. }
    ));
}

#[test]
fn a_zoomed_plot_s_link_shows_the_same_window() {
    let mut sender = plotted_pyramid();
    let shown_name = sender.plotted().variable_info().map(|v| v.name.clone());
    assert_eq!(shown_name.as_deref(), Some("2/t2m"), "a coarse level");
    sender.layers.base.selection_mut().store_target = "https://x.org/p.zarr".into();
    // A quarter of each axis, on level-0 cell edges (64 cells).
    let window = ((0.25, 0.5), (0.5, 0.75));
    sender.nav.heatmap_zoom = 4.0;
    sender.nav.visible_uv = window;
    let link = sender.share_link().unwrap_or_default();
    assert!(link.contains("&view="), "{link}");

    // The receiver has the store open; the link's other actions replay.
    let mut receiver = plotted_pyramid();
    let actions = parse_share(&link).unwrap_or_default();
    receiver.queue_actions(actions[1..].to_vec(), Origin::Link);
    let ctx = egui::Context::default();
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while receiver.nav.pending_view.is_none() {
        assert!(web_time::Instant::now() < deadline, "the view never came");
        receiver.poll_block_prefetch_results();
        receiver.poll_coordinate_results();
        receiver.run_commands(&ctx);
        std::thread::yield_now();
    }
    receiver.apply_pending_view(CANVAS, PlotType::Heatmap);
    assert!(receiver.nav.pending_view.is_none(), "applied");
    let (u, v) = shown(&receiver);
    let cell = 1.0 / 64.0;
    for (got, want) in [(u, window.0), (v, window.1)] {
        assert!(
            (got.0 - want.0).abs() < cell && (got.1 - want.1).abs() < cell,
            "{got:?} for {want:?}, link {link}"
        );
    }
}

#[test]
fn a_view_waiting_for_a_replaced_plot_is_dropped() {
    let mut app = plotted_pyramid();
    app.nav.pending_view = Some(crate::app::state::navigation::PendingView {
        window: ((0.0, 0.5), (0.0, 0.5)),
        plot: (u64::MAX, 0),
    });
    app.apply_pending_view(CANVAS, PlotType::Heatmap);
    assert!(app.nav.pending_view.is_none());
    assert_eq!(app.nav.heatmap_zoom, 1.0, "nothing applied");
}

#[test]
fn a_view_at_the_edge_of_an_odd_level_0_names_its_last_cell() {
    let mut sender = plotted_pyramid();
    let sel = sender.layers.base.selection_mut();
    sel.store_target = "https://x.org/p.zarr".into();
    // Level 0 one cell short of what the coarse level covers, as halving an
    // odd length leaves it (63 cells, the coarse level's last ending at 64).
    for var in sel.metadata.iter_mut().flat_map(|m| m.variables.iter_mut()) {
        if var.name == "0/t2m" {
            var.shape[2] = 63;
        }
        if let Some(r) = var.multiscale.as_mut() {
            Arc::make_mut(&mut r.set).levels[0].shape[2] = 63;
        }
    }
    sender.nav.heatmap_zoom = 4.0;
    sender.nav.visible_uv = ((0.75, 1.0), (0.5, 0.75));
    let link = sender.share_link().unwrap_or_default();
    let actions = parse_share(&link).unwrap_or_default();
    let Some(AppAction::SetView { x_to, .. }) = actions.last() else {
        panic!("no view in {link}");
    };
    // No coordinates in this fixture: cells are named by position.
    assert_eq!(x_to, "#63", "the last cell, not one past it: {link}");
}
