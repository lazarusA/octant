//! Block table tests: per-plane stats, block rows and neighbors.

use super::*;

const DIMS: Dims = Dims { w: 20, h: 9, d: 17 };

fn planes_of(values: &[f32]) -> Vec<[f32; 3]> {
    let grid = grid_of(DIMS);
    let columns = grid.w * grid.h;
    let mut planes = vec![EMPTY; DIMS.d * columns];
    for z in 0..DIMS.d {
        plane_stats(values, DIMS, grid, z, &mut planes[z * columns..][..columns]);
    }
    planes
}

fn own_of(values: &[f32]) -> Vec<[f32; 3]> {
    let grid = grid_of(DIMS);
    let planes = planes_of(values);
    let mut own = vec![EMPTY; grid.w * grid.h * grid.d];
    for bz in 0..grid.d {
        own_row(&planes, DIMS, grid, bz, &mut own);
    }
    own
}

#[test]
fn one_plane_updates_its_block_row() {
    let n = DIMS.w * DIMS.h * DIMS.d;
    let mut values = vec![1.0f32; n];
    let grid = grid_of(DIMS);
    let columns = grid.w * grid.h;
    let mut planes = planes_of(&values);
    let mut own = own_of(&values);
    // A playback step rewrites plane 9 alone (block row 1).
    values[(9 * DIMS.h + 3) * DIMS.w + 4] = -7.0;
    plane_stats(
        &values,
        DIMS,
        grid,
        9,
        &mut planes[9 * columns..][..columns],
    );
    own_row(&planes, DIMS, grid, 1, &mut own);
    assert_eq!(own, own_of(&values));
    assert_eq!(own[columns][0], -7.0);
}

#[test]
fn blocks_hold_their_voxels_range() {
    let n = DIMS.w * DIMS.h * DIMS.d;
    let mut values = vec![0.0f32; n];
    // Voxel (17, 8, 16): block (2, 1, 2), the partial last one on each axis.
    values[(16 * DIMS.h + 8) * DIMS.w + 17] = 5.0;
    values[0] = f32::NAN;
    let grid = grid_of(DIMS);
    assert_eq!((grid.w, grid.h, grid.d), (3, 2, 3));
    let own = own_of(&values);
    let at = |b: [usize; 3]| own[(b[2] * grid.h + b[1]) * grid.w + b[0]];
    assert_eq!(at([2, 1, 2]), [0.0, 5.0, 0.0]);
    assert_eq!(at([0, 0, 0]), [0.0, 0.0, 1.0]);
    // Neighbors see the 5 and the missing voxel; far blocks do not.
    assert_eq!(widened(&own, grid, [1, 0, 1])[..3], [0.0, 5.0, 1.0]);
    assert_eq!(widened(&own, grid, [0, 0, 2])[1], 0.0);
    assert_eq!(widened(&own, grid, [0, 1, 1])[2], 1.0);
    assert_eq!(widened(&own, grid, [2, 1, 2])[2], 0.0);
}

#[test]
fn a_block_without_values_is_empty() {
    let own = own_of(&vec![f32::NAN; DIMS.w * DIMS.h * DIMS.d]);
    assert!(own.iter().all(|b| b[0] > b[1] && b[2] == 1.0));
}
