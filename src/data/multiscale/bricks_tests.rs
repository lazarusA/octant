//! Brick geometry: counts, windows with their apron, the table cells each
//! brick covers (every cell exactly once per level) and children, on an
//! isotropic stack, one with z never downsampled, and a half-cell-centered
//! OME pyramid of odd shape.

use super::bricks::{BRICK, BrickKey, Bricks};
use super::types::{Multiscale, ScaleLevel};

/// A (z, y, x) stack: level `l` of `shapes[l]`, `factors[l]`, `offsets[l]`.
fn stack(shapes: &[[u64; 3]], factors: &[[f64; 3]], offsets: &[[f64; 3]]) -> Multiscale {
    Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: (0..shapes.len())
            .map(|l| ScaleLevel {
                path: format!("{l}"),
                shape: shapes[l].to_vec(),
                factor: factors[l].to_vec(),
                offset: offsets[l].to_vec(),
            })
            .collect(),
    }
}

/// x, y, z are dims 2, 1, 0.
const XYZ: [usize; 3] = [2, 1, 0];

fn iso() -> Multiscale {
    stack(
        &[[256, 256, 256], [128, 128, 128], [64, 64, 64]],
        &[[1.0; 3], [2.0; 3], [4.0; 3]],
        &[[0.0; 3]; 3],
    )
}

/// Every table cell maps to exactly one brick per level, and that brick's
/// `cells` hold it.
fn assert_partition(set: &Multiscale) {
    let bricks = Bricks { set, dims: XYZ };
    let grid = bricks.grid();
    for level in 0..set.len() {
        let counts = bricks.counts(level);
        for i in 0..3 {
            let mut next = 0;
            for b in 0..counts[i] {
                let mut b3 = [0; 3];
                b3[i] = b;
                let r = bricks.cells(BrickKey {
                    level: level as u8,
                    b: b3,
                })[i]
                    .clone();
                assert_eq!(r.start, next, "level {level} axis {i} brick {b}");
                next = r.end;
                for c in r {
                    assert_eq!(bricks.brick_along(level, i, c), b);
                }
            }
            assert_eq!(next, grid[i], "level {level} axis {i} covers the grid");
        }
    }
}

#[test]
fn isotropic_bricks_nest_eight_to_one() {
    let set = iso();
    let bricks = Bricks {
        set: &set,
        dims: XYZ,
    };
    assert_eq!(bricks.grid(), [4, 4, 4]);
    assert_eq!(bricks.counts(2), [1, 1, 1]);
    let root = BrickKey {
        level: 2,
        b: [0; 3],
    };
    assert_eq!(bricks.cells(root), [0..4, 0..4, 0..4]);
    let children: Vec<_> = bricks.children(root).collect();
    assert_eq!(children.len(), 8);
    let child = BrickKey {
        level: 1,
        b: [1, 0, 1],
    };
    assert!(children.contains(&child));
    assert_eq!(bricks.cells(child), [2..4, 0..2, 2..4]);
    assert_eq!(bricks.children(child).count(), 8);
    assert!(
        bricks
            .children(BrickKey {
                level: 0,
                b: [0; 3]
            })
            .next()
            .is_none()
    );
    assert_partition(&set);
}

#[test]
fn windows_are_the_interiors_within_the_level() {
    let set = iso();
    let bricks = Bricks {
        set: &set,
        dims: XYZ,
    };
    let w = bricks
        .window(BrickKey {
            level: 0,
            b: [0, 1, 3],
        })
        .expect("window");
    assert_eq!(w, [0..BRICK, BRICK..2 * BRICK, 3 * BRICK..256]);
}

#[test]
fn z_kept_whole_has_one_child_along_it() {
    // z (dim 0) never downsampled: a brick's children differ in x and y.
    let set = stack(
        &[[100, 256, 256], [100, 128, 128]],
        &[[1.0, 1.0, 1.0], [1.0, 2.0, 2.0]],
        &[[0.0; 3]; 2],
    );
    let bricks = Bricks {
        set: &set,
        dims: XYZ,
    };
    let parent = BrickKey {
        level: 1,
        b: [0, 0, 1],
    };
    let children: Vec<_> = bricks.children(parent).collect();
    assert_eq!(children.len(), 4);
    assert!(children.iter().all(|c| c.b[2] == 1));
    assert_partition(&set);
}

#[test]
fn half_cell_centered_odd_pyramids_partition_the_grid() {
    // Beechnut's shape (z, y, x), levels offset by (f - 1) / 2 cells.
    let set = stack(
        &[
            [1546, 1024, 1024],
            [773, 512, 512],
            [386, 256, 256],
            [193, 128, 128],
        ],
        &[[1.0; 3], [2.0; 3], [4.0; 3], [8.0; 3]],
        &[[0.0; 3], [0.5; 3], [1.5; 3], [3.5; 3]],
    );
    let bricks = Bricks {
        set: &set,
        dims: XYZ,
    };
    assert_eq!(bricks.grid(), [16, 16, 25]);
    assert_eq!(bricks.counts(2), [4, 4, 7]);
    assert_partition(&set);
    // A child's cells lie inside its parent's.
    let parent = BrickKey {
        level: 2,
        b: [1, 2, 6],
    };
    let outer = bricks.cells(parent);
    for child in bricks.children(parent) {
        let inner = bricks.cells(child);
        for i in 0..3 {
            let overlap = inner[i].start.max(outer[i].start) < inner[i].end.min(outer[i].end);
            assert!(overlap, "{child:?} axis {i}: {inner:?} vs {outer:?}");
        }
    }
}

#[test]
fn only_whole_factors_within_half_a_cell_brick_the_table() {
    let shapes = [[256, 256, 256], [128, 128, 128]];
    let aligned = |factor: f64, offset: f64| {
        let set = stack(&shapes, &[[1.0; 3], [factor; 3]], &[[0.0; 3], [offset; 3]]);
        Bricks {
            set: &set,
            dims: XYZ,
        }
        .aligned(2)
    };
    assert!(aligned(2.0, 0.0));
    assert!(aligned(2.0, 0.5), "OME's half-cell centering");
    assert!(aligned(4.0, -64.0), "a whole table cell off");
    // Table cell 1 (level-0 64..128) maps to brick 1 (96..192): 64..96 lies
    // 21 level cells before its apron.
    assert!(!aligned(1.5, 0.0), "a fractional factor");
    assert!(!aligned(2.0, 10.0), "brick edges 5 level cells off");
}
