use super::layout::{COLLAPSE_ORDER, ITEM_COUNT, TopBarItem};

#[test]
fn test_collapse_order_lists_every_item_once() {
    let mut seen = [false; ITEM_COUNT];
    for item in COLLAPSE_ORDER {
        assert!(!seen[item as usize], "{item:?} listed twice");
        seen[item as usize] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

#[test]
fn test_brand_collapses_last() {
    assert_eq!(COLLAPSE_ORDER.last(), Some(&TopBarItem::Brand));
}
