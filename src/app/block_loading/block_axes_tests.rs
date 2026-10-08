//! `block_axes`: a selection's indices and windows inside a block.

use super::block_axes::{dim_bounds, fixed_indices};
use crate::data::octant_block::OctantBlock;
use std::collections::HashMap;

/// A `(time, lat, lon)` block of shape `[4, 3, 5]` at `origin`.
fn block(origin: [usize; 3]) -> OctantBlock {
    OctantBlock::new(
        "v".into(),
        vec![4, 3, 5],
        vec!["time".into(), "lat".into(), "lon".into()],
        origin.to_vec(),
        vec![0.0; 60],
        HashMap::new(),
        HashMap::new(),
    )
}

fn names() -> Vec<String> {
    vec!["time".into(), "lat".into(), "lon".into()]
}

#[test]
fn the_animated_dimension_takes_the_step_others_the_selection() {
    let b = block([8, 0, 10]);
    let fixed = fixed_indices(&b, &names(), Some(0), &[0, 2, 13], 10);
    assert_eq!(
        fixed,
        vec![2, 2, 3],
        "indices are relative to the block origin"
    );
}

#[test]
fn indices_follow_dimension_names_not_positions() {
    let b = block([0, 0, 0]);
    // The variable stores (lon, lat, time); the block is (time, lat, lon).
    let orig = vec!["lon".to_string(), "lat".to_string(), "time".to_string()];
    let fixed = fixed_indices(&b, &orig, None, &[4, 1, 3], 0);
    assert_eq!(fixed, vec![3, 1, 4]);
}

#[test]
fn a_range_inside_the_block_becomes_its_local_window() {
    let b = block([0, 0, 10]);
    let ranges = [(0, 3), (0, 2), (11, 13)];
    assert_eq!(dim_bounds(&b, &names(), 2, &ranges), ((11, 13), (1, 4)));
}

#[test]
fn a_range_past_the_block_end_is_clamped_to_it() {
    let b = block([0, 0, 10]);
    let ranges = [(0, 3), (0, 2), (12, 40)];
    assert_eq!(dim_bounds(&b, &names(), 2, &ranges), ((12, 40), (2, 5)));
}

#[test]
fn a_range_missing_the_block_keeps_the_whole_block() {
    let b = block([0, 0, 10]);
    let ranges = [(0, 3), (0, 2), (0, 4)];
    assert_eq!(dim_bounds(&b, &names(), 2, &ranges), ((0, 4), (0, 5)));
}

#[test]
fn an_unselected_dimension_spans_the_block() {
    let b = block([0, 0, 0]);
    assert_eq!(dim_bounds(&b, &names(), 1, &[]), ((0, 2), (0, 3)));
}
