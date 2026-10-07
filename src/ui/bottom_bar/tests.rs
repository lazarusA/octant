use super::layout::{BottomBarItem, COLLAPSE_ORDER, ITEM_COUNT, LEFT_ITEMS, RIGHT_ITEMS};
use super::timeline::{Timeline, is_display_coord};
use crate::app::OctantApp;
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

#[test]
fn test_display_coord_keeps_dates_and_formats_all_numbers() {
    assert!(is_display_coord("2020-01-01"));
    assert!(is_display_coord("12:30"));
    assert!(is_display_coord("2020-01-01T00:00:00"));
    assert!(!is_display_coord("-5"));
    assert!(!is_display_coord("1e-3"));
    assert!(!is_display_coord("42.5"));
    assert!(!is_display_coord("7"));
    assert!(!is_display_coord("   "));
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
    OctantApp {
        plotted_dataset_metadata: Some(meta),
        plotted_animated_dim: Some(0),
        ..Default::default()
    }
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
