//! A composite brick's apron facing a resident neighbour takes the colors
//! that neighbour shows there: blended from its channels with the plot's
//! stretches, flipped axes included.

use std::collections::HashMap;
use std::sync::Arc;

use super::Lod;
use super::composite::CompositeLod;
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::{BrickKey, STORED};
use crate::data::multiscale::{Multiscale, ScaleLevel};
use crate::data::slicing::composite::ChannelColorConfig;
use crate::plots::volume::encode::packed_rgba;

/// One level of (c, z, y, x) 2 x 64 x 64 x 128: two bricks along x.
fn set() -> Arc<Multiscale> {
    Arc::new(Multiscale {
        name: "v".into(),
        dims: ["c", "z", "y", "x"].map(String::from).to_vec(),
        levels: vec![ScaleLevel {
            path: "0".into(),
            shape: vec![2, 64, 64, 128],
            factor: vec![1.0; 4],
            offset: vec![0.0; 4],
        }],
    })
}

/// The block of brick x = 1 (cells x 64..128): channel 0 rises along x,
/// channel 1 along y.
fn right_block() -> OctantBlock {
    let n = 64 * 64 * 64;
    let values: Vec<f32> = (0..2 * n)
        .map(|i| {
            let (c, j) = (i / n, i % n);
            if c == 0 {
                (j % 64) as f32
            } else {
                (j / 64 % 64) as f32
            }
        })
        .collect();
    OctantBlock::new(
        "v".into(),
        vec![2, 64, 64, 64],
        ["c", "z", "y", "x"].map(String::from).to_vec(),
        vec![0, 0, 0, 64],
        Arc::from(values),
        HashMap::new(),
        HashMap::new(),
    )
    .expect("block")
}

fn configs() -> Vec<ChannelColorConfig> {
    (0..2)
        .map(|i| ChannelColorConfig {
            index: i,
            name: format!("c{i}"),
            color_rgb: [[255, 0, 0], [0, 255, 0]][i],
            visible: true,
            window: None,
            display: Default::default(),
        })
        .collect()
}

#[test]
fn a_composite_apron_shows_its_neighbours_colors() {
    let block = right_block();
    let mut lod = Lod::bare(set(), [3, 2, 1]);
    lod.composite = Some(CompositeLod::channels(0, configs(), &block, (3, 2, 1)));
    let left = BrickKey {
        level: 0,
        b: [0; 3],
    };
    let n = STORED as usize;
    // The left brick's last stored column (x = 65) is the right brick's
    // first cell column.
    let region = [n - 1..n, 0..n, 10..11];
    let fixed = [0; 4];
    let apron = lod
        .colors_over(left, &region, &block, ([3, 2, 1], &fixed))
        .expect("colors");
    let composite = lod.composite.as_ref().expect("composite");
    let whole = composite
        .slice(&block, (3, 2, 1), ((0, 64), (0, 64), (0, 64)), &fixed)
        .expect("whole");
    // Stored row y covers cell y - 1 (clamped at 0), plane z 10 cell 9.
    for (y, got) in apron.iter().enumerate() {
        let cell_y = y.saturating_sub(1).min(63);
        let expected = packed_rgba(whole.values[(9 * 64 + cell_y) * 64]);
        assert_eq!(*got, expected, "stored row {y}");
    }
}
