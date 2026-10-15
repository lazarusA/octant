//! Detail tile placement on level 0's grid and on the plotted quad.

use super::geometry::{Axis, covers, pixels_over};
use crate::data::multiscale::{Multiscale, MultiscaleKind, ScaleLevel};

/// One axis of 64 cells at level 0, 32 at level 1, 16 at level 2.
fn stack() -> Multiscale {
    let levels = (0..3)
        .map(|l| ScaleLevel {
            path: l.to_string(),
            shape: vec![64 >> l],
            factor: vec![f64::from(1u32 << l)],
            offset: vec![0.0],
        })
        .collect();
    Multiscale::new(
        "v".into(),
        vec!["x".into()],
        levels,
        MultiscaleKind::GeoZarr,
    )
    .expect("stack")
}

#[test]
fn the_visible_window_follows_the_quad_and_its_flip() {
    let s = stack();
    let axis = Axis::plotted(&s, 2, 0, (0, 15), false).expect("axis");
    assert_eq!((axis.lo, axis.hi), (0.0, 64.0));
    assert_eq!(axis.visible((0.25, 0.5)), (16.0, 32.0));
    assert_eq!(
        axis.visible((-1.0, 2.0)),
        (0.0, 64.0),
        "clamped to the window"
    );
    let flipped = Axis {
        flipped: true,
        ..axis
    };
    assert_eq!(flipped.visible((0.25, 0.5)), (32.0, 48.0));
}

#[test]
fn tile_cells_widen_align_and_stay_in_the_window() {
    let s = stack();
    let axis = Axis::plotted(&s, 2, 0, (0, 15), false).expect("axis");
    assert_eq!(axis.cells(&s, 0, (16.0, 32.0), 0.25, 1), Some(12..36));
    assert_eq!(axis.cells(&s, 0, (16.0, 32.0), 0.25, 16), Some(0..48));
    assert_eq!(axis.cells(&s, 1, (16.0, 32.0), 0.0, 1), Some(8..16));
    // A plotted window of level-2 cells 8..=11 (level-0 32..48).
    let part = Axis::plotted(&s, 2, 0, (8, 11), false).expect("axis");
    assert_eq!(part.cells(&s, 0, (32.0, 40.0), 0.25, 16), Some(32..48));
}

#[test]
fn tiles_sit_where_their_cells_are_on_the_quad() {
    let s = stack();
    let axis = Axis::plotted(&s, 2, 0, (0, 15), false).expect("axis");
    let cover = axis.cover(&s, 1, &(8..16)).expect("cover");
    assert_eq!(cover, (16.0, 32.0));
    assert_eq!(axis.quad_fraction(cover), (0.25, 0.5));
    let flipped = Axis {
        flipped: true,
        ..axis
    };
    assert_eq!(flipped.quad_fraction(cover), (0.5, 0.75));
    assert!(covers(cover, (16.0, 30.0)));
    assert!(!covers(cover, (15.0, 30.0)));
}

#[test]
fn pixels_count_only_the_canvas_over_the_quad() {
    assert_eq!(pixels_over(1000, (0.2, 0.4)), 1000);
    assert_eq!(pixels_over(1000, (-0.5, 1.5)), 500);
}
