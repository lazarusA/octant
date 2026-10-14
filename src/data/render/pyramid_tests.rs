//! Pyramids read level 0 from the matrix they were built from, and the
//! resampler reuses one tile buffer across zooms.

use super::{AggregationOp, MatrixPyramid, ViewportResampler};
use std::sync::Arc;

/// A `w` x `h` matrix holding `0, 1, 2, ...`.
fn matrix(w: usize, h: usize) -> Vec<f32> {
    (0..w * h).map(|i| i as f32).collect()
}

#[test]
fn level_zero_holds_no_copy_and_samples_read_the_matrix() {
    let values = matrix(8, 6);
    let pyramid = MatrixPyramid::new(&values, 8, 6, "m", AggregationOp::Mean, 2);
    assert!(pyramid.levels.len() > 1, "coarser levels are built");
    assert!(pyramid.levels[0].values.is_empty());
    assert_eq!(pyramid.levels[1].values.len(), 4 * 3);
    // Level 1 averages 2x2 blocks of the matrix: (0 + 1 + 8 + 9) / 4.
    assert_eq!(pyramid.levels[1].values[0], 4.5);

    // The whole matrix at full resolution samples level 0, i.e. the matrix.
    let sample = pyramid.sample_viewport(&values, (0.0, 1.0), (0.0, 1.0), (8, 6));
    assert_eq!((sample.width, sample.height), (8, 6));
    assert_eq!(sample.values, values);
}

#[test]
fn resampling_reuses_its_tile_buffer() {
    let values = matrix(64, 64);
    let pyramid = Arc::new(MatrixPyramid::new(
        &values,
        64,
        64,
        "m",
        AggregationOp::Mean,
        4,
    ));
    let mut resampler = ViewportResampler::new(Some(pyramid));
    let first = resampler
        .resample_if_needed(&values, (0.0, 1.0), (0.0, 1.0), 32, 32)
        .map(|tile| tile.values.as_ptr())
        .expect("first tile");
    // Zoomed in far enough to need another tile.
    let second = resampler
        .resample_if_needed(&values, (0.0, 0.25), (0.0, 0.25), 32, 32)
        .map(|tile| (tile.values.as_ptr(), tile.values.len(), tile.values[0]))
        .expect("second tile");
    assert_eq!(second.0, first, "the same buffer");
    assert_eq!(second.1, 32 * 32);
    assert_eq!(
        second.2, 0.0,
        "the zoomed tile starts at the matrix's corner"
    );
}
