use super::layout::{COLLAPSE_ORDER, ITEM_COUNT, ItemWidths, TopBarItem, compute_compact};

const FULL: f32 = 100.0;
const COMPACT: f32 = 30.0;

fn uniform_widths() -> [ItemWidths; ITEM_COUNT] {
    [ItemWidths {
        full: FULL,
        compact: COMPACT,
    }; ITEM_COUNT]
}

fn compact_count(flags: super::layout::CompactFlags) -> usize {
    COLLAPSE_ORDER.iter().filter(|&&it| flags.get(it)).count()
}

#[test]
fn test_everything_full_when_it_fits() {
    let flags = compute_compact(&uniform_widths(), FULL * ITEM_COUNT as f32);
    assert_eq!(compact_count(flags), 0);
}

#[test]
fn test_everything_compact_when_too_narrow() {
    let flags = compute_compact(&uniform_widths(), 0.0);
    assert_eq!(compact_count(flags), ITEM_COUNT);
}

#[test]
fn test_collapses_one_at_a_time_in_order() {
    let widths = uniform_widths();
    let full_total = FULL * ITEM_COUNT as f32;
    for n in 1..=ITEM_COUNT {
        // Just enough room once exactly `n` items are compact.
        let available = full_total - (FULL - COMPACT) * n as f32;
        let flags = compute_compact(&widths, available);
        assert_eq!(compact_count(flags), n, "available = {available}");
        for (i, &item) in COLLAPSE_ORDER.iter().enumerate() {
            assert_eq!(flags.get(item), i < n, "{item:?} with {n} collapsed");
        }
    }
}

#[test]
fn test_result_fits_whenever_possible() {
    let widths = uniform_widths();
    let min_total = COMPACT * ITEM_COUNT as f32;
    let max_total = FULL * ITEM_COUNT as f32;
    let mut available = min_total;
    while available <= max_total {
        let flags = compute_compact(&widths, available);
        let used: f32 = COLLAPSE_ORDER
            .iter()
            .map(|&it| {
                let w = widths[it as usize];
                if flags.get(it) { w.compact } else { w.full }
            })
            .sum();
        assert!(used <= available, "used {used} > available {available}");
        available += 7.0;
    }
}

#[test]
fn test_absent_status_is_skipped() {
    let mut widths = uniform_widths();
    widths[TopBarItem::Status as usize] = ItemWidths::default();
    let full_total: f32 = widths.iter().map(|w| w.full).sum();
    let flags = compute_compact(&widths, full_total - 1.0);
    // Status saves nothing, so the next item in order must collapse too.
    assert!(flags.get(TopBarItem::Theme));
    assert!(!flags.get(TopBarItem::Cache));
}

#[test]
fn test_brand_collapses_last() {
    assert_eq!(COLLAPSE_ORDER.last(), Some(&TopBarItem::Brand));
}
