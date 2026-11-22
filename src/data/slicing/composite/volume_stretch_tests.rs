//! A composite volume sliced in parts with the stretch its whole recorded
//! colors every part as the whole (bricks of one plot, `app::volume_lod`);
//! stretched over its own values, a part does not.

use std::collections::HashMap;
use std::sync::Arc;

use super::{
    ChannelColorConfig, CompositeStretch, slice_multichannel_volume_stretched,
    slice_rgb_volume_stretched,
};
use crate::data::octant_block::OctantBlock;

/// Three bands over 1 x 2 x 4 (z, y, x), values rising along x.
fn block() -> OctantBlock {
    let values: Vec<f32> = (0..3)
        .flat_map(|band| (0..8).map(move |i| (band * 100 + i * 10) as f32))
        .collect();
    let dims = ["band", "z", "y", "x"].map(String::from).to_vec();
    OctantBlock::new(
        "bands".into(),
        vec![3, 1, 2, 4],
        dims,
        vec![0; 4],
        Arc::from(values),
        HashMap::new(),
        HashMap::new(),
    )
    .expect("block")
}

const DIMS: (usize, usize, usize, usize) = (0, 3, 2, 1);
const WHOLE: ((usize, usize), (usize, usize), (usize, usize)) = ((0, 4), (0, 2), (0, 1));
const HALF: ((usize, usize), (usize, usize), (usize, usize)) = ((0, 2), (0, 2), (0, 1));

/// The first two columns of each row of the whole's packed colors.
fn left_half(values: &[f32]) -> Vec<f32> {
    values.chunks(4).flat_map(|row| row[..2].to_vec()).collect()
}

#[test]
fn rgb_parts_share_the_whole_stretch() {
    let (block, fixed, ch) = (block(), [0; 4], [Some(0), Some(1), Some(2)]);
    let mut stretch = CompositeStretch::default();
    let whole = slice_rgb_volume_stretched(&block, DIMS, WHOLE, &fixed, ch, &mut stretch);
    let whole = whole.expect("whole").values;
    let part = slice_rgb_volume_stretched(&block, DIMS, HALF, &fixed, ch, &mut stretch.clone());
    assert_eq!(part.expect("part").values, left_half(&whole));
    let own = CompositeStretch::default();
    let alone = slice_rgb_volume_stretched(&block, DIMS, HALF, &fixed, ch, &mut own.clone());
    assert_ne!(alone.expect("alone").values, left_half(&whole));
}

#[test]
fn channel_parts_share_the_whole_stretch() {
    let block = block();
    let configs: Vec<ChannelColorConfig> = (0..3)
        .map(|i| ChannelColorConfig {
            index: i,
            name: format!("c{i}"),
            color_rgb: [[255, 0, 0], [0, 255, 0], [0, 0, 255]][i],
            visible: true,
            window: None,
            display: Default::default(),
        })
        .collect();
    let (fixed, mut stretch) = ([0; 4], CompositeStretch::default());
    let slice = |r, s: &mut CompositeStretch| {
        slice_multichannel_volume_stretched(&block, DIMS, r, &fixed, &configs, s)
    };
    let whole = slice(WHOLE, &mut stretch).expect("whole").values;
    let part = slice(HALF, &mut stretch.clone()).expect("part").values;
    assert_eq!(part, left_half(&whole));
    let alone = slice(HALF, &mut CompositeStretch::default())
        .expect("alone")
        .values;
    assert_ne!(alone, left_half(&whole));
}
