//! Level choice from a plot's demand.

use super::pick::{DimDemand, ResolutionDemand, fit_level, pinned_level};
use super::types::{Multiscale, ScaleLevel};

/// `(time, y, x)` at 4x64x64, 4x32x32, 4x16x16, then 2x8x8 (time halved).
fn stack() -> Multiscale {
    let level = |path: &str, shape: [u64; 3], factor: [f64; 3]| ScaleLevel {
        path: path.into(),
        shape: shape.to_vec(),
        factor: factor.to_vec(),
        offset: vec![0.0; 3],
    };
    let levels = vec![
        level("0", [4, 64, 64], [1.0, 1.0, 1.0]),
        level("1", [4, 32, 32], [1.0, 2.0, 2.0]),
        level("2", [4, 16, 16], [1.0, 4.0, 4.0]),
        level("3", [2, 8, 8], [2.0, 8.0, 8.0]),
    ];
    let dims = ["time", "y", "x"].map(String::from).to_vec();
    Multiscale::new("v".into(), dims, levels).expect("stack")
}

fn demand(time: DimDemand, px: u32, max: Option<u64>) -> ResolutionDemand {
    ResolutionDemand {
        dims: vec![time, DimDemand::Pixels(px), DimDemand::Pixels(px)],
        max_elements: max,
    }
}

const WHOLE: [std::ops::Range<u64>; 3] = [0..4, 0..64, 0..64];

#[test]
fn the_finest_level_within_the_pixels_is_read() {
    let s = stack();
    assert_eq!(fit_level(&s, &WHOLE, &demand(DimDemand::Full, 64, None)), 0);
    assert_eq!(fit_level(&s, &WHOLE, &demand(DimDemand::Full, 63, None)), 1);
    assert_eq!(fit_level(&s, &WHOLE, &demand(DimDemand::Full, 20, None)), 2);
    // A smaller window reads finer.
    let part = [0..4, 0..16, 0..16];
    assert_eq!(fit_level(&s, &part, &demand(DimDemand::Full, 20, None)), 0);
}

#[test]
fn whole_dimensions_exclude_levels_downsampling_them() {
    let s = stack();
    // Level 3 halves time: never read while time is kept whole.
    assert_eq!(fit_level(&s, &WHOLE, &demand(DimDemand::Full, 4, None)), 2);
    assert_eq!(fit_level(&s, &WHOLE, &demand(DimDemand::Any, 8, None)), 3);
}

#[test]
fn the_cell_budget_coarsens() {
    let s = stack();
    let budget = Some(4 * 32 * 32);
    assert_eq!(
        fit_level(&s, &WHOLE, &demand(DimDemand::Full, 1000, budget)),
        1
    );
    assert_eq!(fit_level(&s, &WHOLE, &ResolutionDemand::finest(3)), 0);
}

#[test]
fn pinned_levels_hold_within_the_budget() {
    let s = stack();
    let free = demand(DimDemand::Full, 20, None);
    assert_eq!(pinned_level(&s, 0, &WHOLE, &free), 0);
    assert_eq!(pinned_level(&s, 1, &WHOLE, &free), 1);
    assert_eq!(pinned_level(&s, 9, &WHOLE, &free), 2, "level 3 halves time");
    let tight = demand(DimDemand::Full, 20, Some(4 * 16 * 16));
    assert_eq!(pinned_level(&s, 0, &WHOLE, &tight), 2);
}
