use super::layout::{BottomBarItem, COLLAPSE_ORDER, ITEM_COUNT, LEFT_ITEMS, RIGHT_ITEMS};
use super::timeline::is_display_coord;

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
fn test_display_coord_keeps_dates_and_formats_bare_numbers() {
    assert!(is_display_coord("2020-01-01"));
    assert!(is_display_coord("12:30"));
    assert!(is_display_coord("-5"));
    assert!(!is_display_coord("42.5"));
    assert!(!is_display_coord("7"));
    assert!(!is_display_coord("   "));
}
