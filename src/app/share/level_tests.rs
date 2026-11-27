//! The multiscale level through a link (`multiscale_level`): `n` pinned,
//! `auto:n` Auto from the level its sender saw; replayed as `SetLevel`
//! before Plot.

use super::parse_share;
use crate::app::OctantApp;
use crate::app::actions::{AppAction, Origin};
use crate::app::layers::LevelChoice;
use crate::app::test_support::{memory_app, poll_until, pyramid_app, selection_of};

/// The pyramid's `0/t2m` plotted at `level` on a `px` canvas.
fn plotted_pyramid(level: LevelChoice, px: u32) -> OctantApp {
    let (mut app, meta) = pyramid_app();
    app.layout.canvas_px = [px, px];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.selected.level = level;
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    app
}

/// `app`'s share link, as if its in-memory store were remote (a local store
/// makes no link).
fn link_of(app: &mut OctantApp) -> String {
    let target = &mut app.layers.base.selection_mut().store_target;
    let local = std::mem::replace(target, "https://x.org/p.zarr".into());
    let link = app.share_link().unwrap_or_default();
    app.layers.base.selection_mut().store_target = local;
    link
}

fn shown_name(app: &OctantApp) -> Option<String> {
    app.plotted().variable_info().map(|v| v.name.clone())
}

fn set_level(level: Option<usize>, auto: bool) -> AppAction {
    AppAction::SetLevel { level, auto }
}

/// Replays `link` (but its OpenSource) on `app` until `0/t2m` shows as
/// `name`.
fn replay_until_shown(app: &mut OctantApp, link: &str, name: &str) {
    let actions = parse_share(link).unwrap_or_default();
    app.queue_actions(actions[1..].to_vec(), Origin::Link);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while shown_name(app).as_deref() != Some(name) {
        assert!(web_time::Instant::now() < deadline, "{name} never shown");
        app.poll_block_prefetch_results();
        app.poll_coordinate_results();
        app.run_commands();
        std::thread::yield_now();
    }
    poll_until(app, |app| app.layers.base.data.matrix.is_some());
}

#[test]
fn multiscale_level_reads_into_set_level_after_the_variable() {
    let read = |value: &str| {
        let link = format!("?src=https://x.org/a&plot=volume&multiscale_level={value}&var=t");
        let actions = parse_share(&link).unwrap_or_default();
        assert_eq!(actions[1], AppAction::SelectVariable { path: "t".into() });
        assert!(matches!(actions.last(), Some(AppAction::Plot { .. })));
        actions
            .iter()
            .find(|a| matches!(a, AppAction::SetLevel { .. }))
            .cloned()
    };
    assert_eq!(read("2"), Some(set_level(Some(2), false)));
    assert_eq!(read("auto:1"), Some(set_level(Some(1), true)));
    assert_eq!(read("auto"), Some(set_level(None, true)));
    for bad in ["coarse", "auto:", "auto:x", "-1"] {
        assert_eq!(read(bad), None, "{bad}");
    }
}

#[test]
fn a_dimension_called_level_stays_a_dimension() {
    let actions = parse_share("?src=https://x.org/a&var=t&level=850").unwrap_or_default();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, AppAction::SetLevel { .. }))
    );
    assert!(actions.iter().any(|a| matches!(
        a,
        AppAction::SetDim { dim: Some(d), value } if d == "level" && value == "850"
    )));
}

#[test]
fn a_link_names_the_level_shown_pinned_or_auto() {
    let mut auto = plotted_pyramid(LevelChoice::Auto, 20);
    assert_eq!(
        shown_name(&auto).as_deref(),
        Some("2/t2m"),
        "a coarse level"
    );
    let link = link_of(&mut auto);
    assert!(
        link.contains("&var=0/t2m&multiscale_level=auto:2"),
        "{link}"
    );

    let mut pinned = plotted_pyramid(LevelChoice::Pinned(1), 20);
    assert_eq!(shown_name(&pinned).as_deref(), Some("1/t2m"));
    let link = link_of(&mut pinned);
    assert!(link.contains("&var=0/t2m&multiscale_level=1"), "{link}");
}

#[test]
fn a_pinned_plot_s_link_pins_the_same_level() {
    let link = link_of(&mut plotted_pyramid(LevelChoice::Pinned(1), 20));
    let mut receiver = plotted_pyramid(LevelChoice::Auto, 20);
    replay_until_shown(&mut receiver, &link, "1/t2m");
    assert_eq!(receiver.plotted().level, LevelChoice::Pinned(1));
    assert_eq!(link_of(&mut receiver), link);
}

#[test]
fn an_auto_link_gives_at_least_the_sender_s_detail_and_stays_auto() {
    // A sender on a large canvas read level 0; a small receiver, which
    // would read level 2 on its own, starts at the sender's level 0.
    let mut fine = plotted_pyramid(LevelChoice::Auto, 400);
    assert_eq!(shown_name(&fine).as_deref(), Some("0/t2m"));
    let link = link_of(&mut fine);
    assert!(link.contains("multiscale_level=auto:0"), "{link}");
    let mut small = plotted_pyramid(LevelChoice::Auto, 20);
    assert_eq!(shown_name(&small).as_deref(), Some("2/t2m"));
    replay_until_shown(&mut small, &link, "0/t2m");
    let level = small.plotted().level;
    assert_eq!(level, LevelChoice::AutoFrom(0));
    assert!(
        level.is_auto(),
        "detail tiles and volume bricks still refine it"
    );

    // A sender on a small canvas never makes a large receiver coarser.
    let mut coarse = plotted_pyramid(LevelChoice::Auto, 20);
    let link = link_of(&mut coarse);
    let mut large = plotted_pyramid(LevelChoice::Auto, 400);
    let actions = parse_share(&link).unwrap_or_default();
    large.queue_actions(actions[1..].to_vec(), Origin::Link);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while large.plotted().level != LevelChoice::AutoFrom(2) {
        assert!(
            web_time::Instant::now() < deadline,
            "the link never applied"
        );
        large.poll_block_prefetch_results();
        large.poll_coordinate_results();
        large.run_commands();
        std::thread::yield_now();
    }
    poll_until(&mut large, |app| !app.new_plot_pending());
    assert_eq!(
        shown_name(&large).as_deref(),
        Some("0/t2m"),
        "its own finer fit"
    );
}

#[test]
fn a_link_s_level_waits_for_the_link_s_plot() {
    // The variable shown: a voice SetLevel plots again, a link's only stages.
    let mut app = plotted_pyramid(LevelChoice::Auto, 20);
    app.queue_actions(vec![set_level(Some(1), false)], Origin::Link);
    app.run_commands();
    assert_eq!(app.selected.level, LevelChoice::Pinned(1));
    assert!(!app.new_plot_pending(), "the link's Plot reads it");

    let mut app = plotted_pyramid(LevelChoice::Auto, 20);
    let colormap = AppAction::SetColormap {
        name: "matplotlib:magma".into(),
        layer: None,
    };
    // Voice commands (the bar's own `run_commands` drops them while the bar
    // is closed, so resume them directly).
    app.queue_actions(vec![set_level(Some(1), false), colormap], Origin::Voice);
    let before = app.layers.base.color.colormap;
    let done = app.resume_commands();
    assert_eq!(done.len(), 1, "the colormap waits: {done:?}");
    assert!(app.new_plot_pending(), "the voice SetLevel plotted again");
    assert_eq!(app.layers.base.color.colormap, before, "held for the block");
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.layers.base.color.colormap == before {
        assert!(
            web_time::Instant::now() < deadline,
            "the colormap never applied"
        );
        app.poll_block_prefetch_results();
        app.resume_commands();
        std::thread::yield_now();
    }
    assert_eq!(shown_name(&app).as_deref(), Some("1/t2m"));
}

#[test]
fn set_level_clamps_and_keeps_auto_without_a_pyramid() {
    let mut app = plotted_pyramid(LevelChoice::Auto, 20);
    let done = app.dispatch(set_level(Some(9), false));
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(app.selected.level, LevelChoice::Pinned(2), "the coarsest");
    let done = app.dispatch(set_level(Some(9), true));
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(app.selected.level, LevelChoice::AutoFrom(2));
    let auto = app.dispatch(set_level(None, false));
    assert!(auto.is_ok(), "{auto:?}");
    assert_eq!(app.selected.level, LevelChoice::Auto);

    let (mut flat, meta) = memory_app();
    let name = meta
        .variables
        .first()
        .map(|v| v.name.clone())
        .unwrap_or_default();
    flat.selected = selection_of(&mut flat, &meta, &name);
    let done = flat.dispatch(set_level(Some(2), false));
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(flat.selected.level, LevelChoice::Auto);
}
