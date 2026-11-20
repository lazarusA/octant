use super::layout::{BottomBarItem, COLLAPSE_ORDER, ITEM_COUNT, LEFT_ITEMS, RIGHT_ITEMS};
use super::timeline::Timeline;
use crate::app::OctantApp;
use crate::app::VariableSelection;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

#[test]
fn test_left_and_right_cover_every_item_once() {
    let mut seen = [false; ITEM_COUNT];
    for item in LEFT_ITEMS.into_iter().chain(RIGHT_ITEMS) {
        assert!(!seen[item as usize], "{item:?} placed twice");
        seen[item as usize] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

#[test]
fn test_badges_hide_before_buttons_collapse() {
    let first_button = COLLAPSE_ORDER
        .iter()
        .position(|i| !i.hides_when_compact())
        .unwrap_or(COLLAPSE_ORDER.len());
    assert!(
        COLLAPSE_ORDER[..first_button]
            .iter()
            .all(|i| i.hides_when_compact())
    );
    assert_eq!(COLLAPSE_ORDER.last(), Some(&BottomBarItem::PlayPause));
}

#[test]
fn test_step_buttons_never_collapse() {
    for item in [
        BottomBarItem::First,
        BottomBarItem::Prev,
        BottomBarItem::Next,
        BottomBarItem::Last,
    ] {
        assert!(!COLLAPSE_ORDER.contains(&item));
    }
}

/// An app animating a 3-step `region` dimension labelled by `labels`.
fn regions_app(labels: &[&str]) -> OctantApp {
    let var = VariableInfo {
        name: "pop".into(),
        shape: vec![3],
        dimension_names: vec!["region".into()],
        ..Default::default()
    };
    let regions = CoordValues::from_labels(labels.iter().map(|s| s.to_string()).collect());
    let meta = DatasetMetadata {
        variables: vec![var],
        dimension_coordinates: regions
            .map(|r| std::collections::HashMap::from([("region".to_string(), r)]))
            .unwrap_or_default(),
        ..Default::default()
    };
    let mut app = OctantApp::default();
    *app.layers.base.selection_mut() = VariableSelection {
        metadata: Some(meta),
        dim_config: vec![crate::app::DimConfig {
            animation: crate::app::AnimationRole::Animated,
            ..Default::default()
        }],
        ..Default::default()
    };
    app
}

#[test]
fn timeline_reads_labels_only_when_there_is_one_per_step() {
    let ctx = egui::Context::default();
    let full = Timeline::cached(&regions_app(&["north", "south", "east"]), &ctx);
    assert_eq!((full.start.as_str(), full.end.as_str()), ("north", "east"));

    let ctx = egui::Context::default();
    let short = Timeline::cached(&regions_app(&["north", "south"]), &ctx);
    assert_ne!(short.end, "south", "two labels do not name three steps");
    assert_ne!(short.start, "north");
}

#[test]
fn the_play_button_shows_buffering_progress_while_it_waits() {
    use super::items::{LabelBuf, label_for};
    let mut app = OctantApp::default();
    let mut buf: LabelBuf = [0; 64];
    app.playback.is_playing = true;
    assert_eq!(label_for(&mut buf, BottomBarItem::PlayPause, &app), "");
    app.playback.buffering = true;
    app.playback.buffer_progress = (12, 45);
    crate::i18n::with_language(crate::i18n::Language::English, || {
        let label = label_for(&mut buf, BottomBarItem::PlayPause, &app);
        assert_eq!(label, "Buffering 12/45");
    });
}

#[test]
fn arrows_walk_the_buttons_up_to_the_timeline() {
    use crate::ui::keyboard::{self, focus};
    use crate::ui::test_input::Harness;
    use egui::{Key, Modifiers, Rect, pos2};

    fn run(app: &mut OctantApp) -> impl FnMut(&mut egui::Ui) + '_ {
        |ui| {
            keyboard::dispatch(app, ui.ctx(), true);
            super::show_bottom_bar(app, ui);
            focus::end_frame(ui.ctx());
        }
    }
    let mut app = regions_app(&["north", "south", "east"]);
    let mut h = Harness::new(Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 300.0)));
    h.settle(2, &mut run(&mut app));
    let mut press = |key| {
        let at = h.press(key, Modifiers::NONE, &mut run(&mut app));
        at.expect("a widget is focused").rect
    };
    // F6 from nowhere enters the first region drawn: this bar.
    let mut rects = vec![press(Key::F6)];
    rects.extend((0..8).map(|_| press(Key::ArrowRight)));
    // The buttons step right up to the timeline, whose arrows then move it.
    let slider = rects
        .iter()
        .position(|r| r.width() > 100.0)
        .expect("the timeline");
    assert!(
        rects[..=slider]
            .windows(2)
            .all(|w| w[0].left() < w[1].left()),
        "{rects:?}"
    );
    assert!(
        rects[slider..].iter().all(|r| *r == rects[slider]),
        "{rects:?}"
    );
}

#[test]
fn timeline_dates_match_the_hover_card() {
    // Hourly CF time coordinate; the plotted variable's own units are "K".
    let time = VariableInfo {
        name: "time".into(),
        shape: vec![3],
        dimension_names: vec!["time".into()],
        units: Some("hours since 2020-01-01".into()),
        ..Default::default()
    };
    let var = VariableInfo {
        name: "t2m".into(),
        shape: vec![3],
        dimension_names: vec!["time".into()],
        units: Some("K".into()),
        ..Default::default()
    };
    let meta = DatasetMetadata {
        variables: vec![var, time],
        dimension_coordinates: std::collections::HashMap::from([(
            "time".to_string(),
            CoordValues::Values(vec![0.0, 6.0, 12.0].into()),
        )]),
        ..Default::default()
    };
    let hover = |i| {
        crate::ui::hover::format::format_dimension_coord(
            Some(&meta),
            meta.variables.first(),
            Some(""),
            "time",
            i,
            3,
            None,
        )
        .value
    };
    let (start, end) = (hover(0), hover(2));
    let mut app = OctantApp::default();
    *app.layers.base.selection_mut() = VariableSelection {
        metadata: Some(meta),
        dim_config: vec![crate::app::DimConfig {
            animation: crate::app::AnimationRole::Animated,
            ..Default::default()
        }],
        ..Default::default()
    };
    let tl = Timeline::cached(&app, &egui::Context::default());
    assert_eq!(
        (tl.start.as_str(), tl.end.as_str()),
        (start.as_str(), end.as_str())
    );
    assert_ne!(tl.start, tl.end, "6-hourly steps are told apart");
}
