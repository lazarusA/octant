use super::{BarItem, CompactFlags, ItemWidths, compute_compact};

/// Minimal bar for exercising the shared algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    A,
    B,
    C,
    D,
    Absent,
}

impl BarItem for Item {
    fn index(self) -> usize {
        self as usize
    }
}

const ITEM_COUNT: usize = 5;
const COLLAPSE_ORDER: [Item; ITEM_COUNT] = [Item::Absent, Item::D, Item::C, Item::B, Item::A];

const FULL: f32 = 100.0;
const COMPACT: f32 = 30.0;

fn uniform_widths() -> [ItemWidths; ITEM_COUNT] {
    [ItemWidths {
        full: FULL,
        compact: COMPACT,
    }; ITEM_COUNT]
}

fn compact_count(flags: CompactFlags<ITEM_COUNT>) -> usize {
    COLLAPSE_ORDER.iter().filter(|&&it| flags.get(it)).count()
}

#[test]
fn test_everything_full_when_it_fits() {
    let flags = compute_compact(&uniform_widths(), &COLLAPSE_ORDER, FULL * ITEM_COUNT as f32);
    assert_eq!(compact_count(flags), 0);
}

#[test]
fn test_everything_compact_when_too_narrow() {
    let flags = compute_compact(&uniform_widths(), &COLLAPSE_ORDER, 0.0);
    assert_eq!(compact_count(flags), ITEM_COUNT);
}

#[test]
fn test_collapses_one_at_a_time_in_order() {
    let widths = uniform_widths();
    let full_total = FULL * ITEM_COUNT as f32;
    for n in 1..=ITEM_COUNT {
        // Just enough room once exactly `n` items are compact.
        let available = full_total - (FULL - COMPACT) * n as f32;
        let flags = compute_compact(&widths, &COLLAPSE_ORDER, available);
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
        let flags = compute_compact(&widths, &COLLAPSE_ORDER, available);
        let used: f32 = COLLAPSE_ORDER
            .iter()
            .map(|&it| {
                let w = widths[it.index()];
                if flags.get(it) { w.compact } else { w.full }
            })
            .sum();
        assert!(used <= available, "used {used} > available {available}");
        available += 7.0;
    }
}

#[test]
fn test_absent_item_is_skipped() {
    let mut widths = uniform_widths();
    widths[Item::Absent.index()] = ItemWidths::default();
    let full_total: f32 = widths.iter().map(|w| w.full).sum();
    let flags = compute_compact(&widths, &COLLAPSE_ORDER, full_total - 1.0);
    // The absent item saves nothing, so the next item in order must collapse.
    assert!(flags.get(Item::D));
    assert!(!flags.get(Item::C));
}

#[test]
fn test_hidden_items_free_their_full_width() {
    let mut widths = uniform_widths();
    widths[Item::D.index()] = ItemWidths {
        full: FULL,
        compact: 0.0,
    };
    let full_total: f32 = widths.iter().map(|w| w.full).sum();
    // Hiding D frees all of its width, so nothing else needs to collapse.
    let flags = compute_compact(&widths, &COLLAPSE_ORDER, full_total - FULL);
    assert!(flags.get(Item::D));
    assert!(!flags.get(Item::C));
}

#[test]
fn test_fixed_widths_never_shrink() {
    let w = ItemWidths::fixed(24.0);
    assert_eq!(w.full, w.compact);
}
