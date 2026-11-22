//! A finer brick's values widen its resident ancestors' ranges, so a coarse
//! brick that looked empty (a mean pyramid averages a bright spot away) is
//! refined once a finer one shows what it holds.

use std::sync::Arc;

use super::Shown;
use super::{Lod, Placed};
use crate::data::BlockCacheKey;
use crate::data::multiscale::bricks::{BrickKey, Bricks};
use crate::data::multiscale::{Multiscale, ScaleLevel};

/// Two levels of (z, y, x) 128^3 and 64^3: eight bricks under one.
fn set() -> Arc<Multiscale> {
    let level = |l: u32| ScaleLevel {
        path: l.to_string(),
        shape: vec![128 >> l; 3],
        factor: vec![f64::from(1 << l); 3],
        offset: vec![0.0; 3],
    };
    Arc::new(Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: vec![level(0), level(1)],
    })
}

fn placed(range: (f32, f32)) -> Placed {
    let block = BlockCacheKey {
        source_id: "s".into(),
        variable_name: "v".into(),
        selections: Vec::new(),
    };
    Placed {
        block,
        range,
        missing: false,
        subs: Vec::new(),
    }
}

#[test]
fn a_finer_brick_widens_its_ancestors_range() {
    let mut lod = Lod::bare(set(), [2, 1, 0]);
    let root = BrickKey {
        level: 1,
        b: [0; 3],
    };
    let child = BrickKey {
        level: 0,
        b: [1, 0, 1],
    };
    let set = lod.set.clone();
    let bricks = Bricks {
        set: &set,
        dims: lod.dims,
    };
    lod.residency
        .insert(&bricks, root, placed((0.0, 0.0)), 0, true);
    let shown = Shown {
        lo: 50.0,
        hi: 100.0,
        nan: false,
        floor: 0.0,
    };
    assert!(
        lod.draws_nothing(root, shown),
        "the mean level is background"
    );
    lod.widen_ancestors(child, (5.0, 80.0), None);
    let range = lod.residency.get(&root).map(|p| p.range);
    assert_eq!(range, Some((0.0, 80.0)));
    assert!(!lod.draws_nothing(root, shown), "refined once seen");
}

/// Three levels of 256^3, the coarsest offset by -64 level-0 cells: the
/// level-1 brick over table cells 2..4 along x straddles level-2 bricks 0
/// and 1 (their edge at level-0 cell 192).
fn offset_set() -> Arc<Multiscale> {
    let level = |l: u32, shape: u64, offset: f64| ScaleLevel {
        path: l.to_string(),
        shape: vec![shape; 3],
        factor: vec![f64::from(1 << l); 3],
        offset: vec![offset; 3],
    };
    Arc::new(Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: vec![level(0, 256, 0.0), level(1, 128, 0.0), level(2, 128, -64.0)],
    })
}

#[test]
fn every_parent_of_a_brick_is_widened() {
    let mut lod = Lod::bare(offset_set(), [2, 1, 0]);
    let set = lod.set.clone();
    let bricks = Bricks {
        set: &set,
        dims: lod.dims,
    };
    let child = BrickKey {
        level: 1,
        b: [1, 0, 0],
    };
    let cells = bricks.cells(child);
    let mut parents: Vec<BrickKey> = [cells[0].start, cells[0].end - 1]
        .map(|x| bricks.cell_brick(2, [x, cells[1].start, cells[2].start]))
        .to_vec();
    parents.dedup();
    assert_eq!(parents.len(), 2, "two parents along x: {parents:?}");
    for &p in &parents {
        lod.residency
            .insert(&bricks, p, placed((0.0, 10.0)), 0, true);
    }
    lod.widen_ancestors(child, (5.0, 80.0), None);
    for p in parents {
        let range = lod.residency.get(&p).map(|r| r.range);
        assert_eq!(range, Some((0.0, 80.0)), "{p:?}");
    }
}

#[test]
fn a_parent_placed_after_its_child_takes_the_childs_range() {
    let mut lod = Lod::bare(set(), [2, 1, 0]);
    let set = lod.set.clone();
    let bricks = Bricks {
        set: &set,
        dims: lod.dims,
    };
    let root = BrickKey {
        level: 1,
        b: [0; 3],
    };
    let child = BrickKey {
        level: 0,
        b: [1, 0, 1],
    };
    // The child's read finished first.
    lod.residency
        .insert(&bricks, child, placed((5.0, 80.0)), 0, false);
    lod.residency
        .insert(&bricks, root, placed((0.0, 10.0)), 0, true);
    lod.absorb_descendants(root);
    let range = lod.residency.get(&root).map(|p| p.range);
    assert_eq!(range, Some((0.0, 80.0)));
    let shown = Shown {
        lo: 50.0,
        hi: 100.0,
        nan: false,
        floor: 0.0,
    };
    assert!(
        !lod.draws_nothing(root, shown),
        "the bright child keeps it split"
    );
}

#[test]
fn only_background_or_missing_bricks_count_as_drawing_nothing() {
    let mut lod = Lod::bare(set(), [2, 1, 0]);
    let root = BrickKey {
        level: 1,
        b: [0; 3],
    };
    let set = lod.set.clone();
    let bricks = Bricks {
        set: &set,
        dims: lod.dims,
    };
    let shown = Shown {
        lo: 100.0,
        hi: 300.0,
        nan: false,
        floor: 0.0,
    };
    // Puncta of 300 averaged to 5 at this level: finer bricks may show them.
    lod.residency
        .insert(&bricks, root, placed((0.0, 5.0)), 0, true);
    assert!(!lod.draws_nothing(root, shown), "a dim brick is split");
    let missing = Placed {
        missing: true,
        ..placed((f32::INFINITY, f32::NEG_INFINITY))
    };
    let hole = BrickKey {
        level: 0,
        b: [1, 0, 1],
    };
    lod.residency.insert(&bricks, hole, missing, 0, false);
    assert!(lod.draws_nothing(hole, shown), "all missing");
    let nan = Shown { nan: true, ..shown };
    assert!(
        !lod.draws_nothing(hole, nan),
        "missing drawn in the NaN color"
    );
}
