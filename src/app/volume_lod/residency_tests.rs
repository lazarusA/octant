//! The indirection table's mirror over a 256^3 stack of 3 levels (a 4^3
//! table) with two atlas slots: a coarse brick fills its cells, a finer one
//! replaces it over its own, slots are reused only from bricks the latest
//! selection no longer wants and never from a pinned one, and evicting the
//! finer brick brings the coarse one back.

use super::residency::Residency;
use crate::data::multiscale::bricks::{BrickKey, Bricks};
use crate::data::multiscale::{Multiscale, ScaleLevel};

fn iso() -> Multiscale {
    Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: (0..3)
            .map(|l| ScaleLevel {
                path: format!("{l}"),
                shape: vec![256 >> l; 3],
                factor: vec![f64::from(1 << l); 3],
                offset: vec![0.0; 3],
            })
            .collect(),
    }
}

fn block(name: &'static str) -> &'static str {
    name
}

fn levels(r: &Residency<&str>) -> Vec<u8> {
    r.table_region(&[0..4, 0..4, 0..4])
        .iter()
        .map(|e| e[3])
        .collect()
}

#[test]
fn finer_bricks_replace_coarser_and_hand_back_on_eviction() {
    let set = iso();
    let bricks = Bricks {
        set: &set,
        dims: [2, 1, 0],
    };
    let mut r = Residency::new(bricks.grid(), [2, 1, 1], 3);
    let root = BrickKey {
        level: 2,
        b: [0; 3],
    };
    let (slot, changed) = r
        .insert(&bricks, root, block("root"), 0, true)
        .expect("free");
    assert_eq!(slot, [0, 0, 0]);
    assert_eq!(changed, vec![[0..4, 0..4, 0..4]]);
    assert!(
        levels(&r).iter().all(|&w| w == 3),
        "the root's level + 1 everywhere"
    );

    let a = BrickKey {
        level: 1,
        b: [0; 3],
    };
    let (slot, _) = r.insert(&bricks, a, block("a"), 0, false).expect("free");
    assert_eq!(slot, [1, 0, 0]);
    let fine = levels(&r).iter().filter(|&&w| w == 2).count();
    assert_eq!(fine, 8, "brick a's 2^3 cells");
    let (key, read) = r.lookup(&bricks, [1, 1, 1]).expect("resident");
    assert_eq!((key, *read), (a, "a"));
    let (key, _) = r.lookup(&bricks, [3, 3, 3]).expect("resident");
    assert_eq!(key, root);

    // No free slot: the root is pinned, `a` wanted by this selection.
    let b = BrickKey {
        level: 1,
        b: [1, 1, 1],
    };
    assert!(r.insert(&bricks, b, block("b"), 0, false).is_none());
    // Selection 1 wants `a` again: still kept.
    r.touch(&a, 1);
    assert!(r.insert(&bricks, b, block("b"), 1, false).is_none());
    // Selection 2 does not: its slot goes to `b`.
    let (slot, changed) = r
        .insert(&bricks, b, block("b"), 2, false)
        .expect("a evicted");
    assert_eq!(slot, [1, 0, 0]);
    assert_eq!(changed, vec![[0..2, 0..2, 0..2], [2..4, 2..4, 2..4]]);
    assert!(!r.holds(&a) && r.holds(&b));
    assert_eq!(r.lookup(&bricks, [1, 1, 1]).map(|(k, _)| k), Some(root));
    assert_eq!(r.lookup(&bricks, [3, 3, 3]).map(|(k, _)| k), Some(b));
}

#[test]
fn table_cells_never_overflow() {
    use crate::data::multiscale::bricks::table_cells;
    assert_eq!(table_cells([576, 87, 100]), Some(5_011_200));
    assert_eq!(table_cells([u32::MAX; 3]), None);
}
