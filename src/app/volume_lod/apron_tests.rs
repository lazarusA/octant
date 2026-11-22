//! Aprons from neighbours: a brick placed beside a resident neighbour
//! whose block is cached takes that neighbour's cells into the apron facing
//! it, and keeps its own edge elsewhere.

use std::sync::Arc;

use super::Lod;
use super::Placed;
use super::encode::{Cube, encode_value, has_missing};
use crate::data::blocks::BlockCache;
use crate::data::multiscale::bricks::{BrickKey, STORED};
use crate::data::multiscale::{Multiscale, ScaleLevel};
use crate::data::{BlockCacheKey, OctantBlock};

/// One level of (z, y, x) 64 x 64 x 128: two bricks along x.
fn set() -> Arc<Multiscale> {
    Arc::new(Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: vec![ScaleLevel {
            path: "0".into(),
            shape: vec![64, 64, 128],
            factor: vec![1.0; 3],
            offset: vec![0.0; 3],
        }],
    })
}

/// The block of brick x = 1 (cells x 64..128), each value its x.
fn right_block() -> OctantBlock {
    block_of((0..64 * 64 * 64).map(|i| (64 + i % 64) as f32).collect())
}

/// Brick x = 1's block holding `values` (z, y, x order).
fn block_of(values: Vec<f32>) -> OctantBlock {
    let dims = ["z", "y", "x"].map(String::from).to_vec();
    OctantBlock::new(
        "v".into(),
        vec![64, 64, 64],
        dims,
        vec![0, 0, 64],
        values,
        Default::default(),
        std::collections::HashMap::new(),
    )
    .expect("block")
}

/// The left brick's cube stitched beside a resident right brick whose
/// block (cached) is `block`.
fn stitched_beside(block: OctantBlock) -> Vec<u16> {
    let mut lod = Lod::bare(set(), [2, 1, 0]);
    let right = BrickKey {
        level: 0,
        b: [1, 0, 0],
    };
    let key = BlockCacheKey {
        source_id: "s".into(),
        variable_name: "v".into(),
        selections: Vec::new(),
    };
    let mut cache = BlockCache::new(1 << 30);
    cache.put(key.clone(), block);
    let bricks = crate::data::multiscale::bricks::Bricks {
        set: &lod.set,
        dims: lod.dims,
    };
    let placed = Placed {
        block: key,
        range: (64.0, 127.0),
        missing: false,
        subs: Vec::new(),
    };
    lod.residency
        .insert(&bricks, right, placed, 0, false)
        .expect("slot");

    let n = STORED as usize;
    let blank = encode_value(-1.0, 0.0, 1.0);
    let mut cube = Cube::Values(vec![blank; n * n * n]);
    let left = BrickKey {
        level: 0,
        b: [0; 3],
    };
    lod.stitch_own(left, &mut cube, &cache, None);
    let Cube::Values(cube) = cube else {
        panic!("a scalar cube");
    };
    cube
}

#[test]
fn an_apron_takes_its_resident_neighbours_cells() {
    let cube = stitched_beside(right_block());
    let n = STORED as usize;
    let blank = encode_value(-1.0, 0.0, 1.0);
    let at = |x: usize, y: usize, z: usize| cube[(z * n + y) * n + x];
    // The last stored column is cell 64, the right brick's first.
    assert_eq!(at(n - 1, 10, 10), encode_value(64.0, 0.0, 1.0));
    assert_eq!(at(n - 1, 0, 10), encode_value(64.0, 0.0, 1.0), "edges too");
    // No brick before x = 0: the first column stays.
    assert_eq!(at(0, 10, 10), blank);
    assert_eq!(at(10, 10, 10), blank, "the interior is the brick's own");
}

#[test]
fn a_missing_neighbour_cell_in_the_apron_marks_the_brick_missing() {
    // The right brick's first column (cell x = 64) is missing.
    let values = (0..64 * 64 * 64)
        .map(|i| if i % 64 == 0 { f32::NAN } else { 1.0 })
        .collect();
    let cube = stitched_beside(block_of(values));
    assert!(
        has_missing(&Cube::Values(cube)),
        "filtered as missing there"
    );
    let whole = stitched_beside(right_block());
    assert!(!has_missing(&Cube::Values(whole)));
}
