//! Mathematical helper functions and numerical reductions.

/// Computes the finite minimum and maximum values in a slice of `f32`s, filtering out NaNs and infinities.
/// Returns `(0.0, 1.0)` as fallback if no valid finite values are present.
pub fn compute_finite_min_max(values: &[f32]) -> (f32, f32) {
    let (lo, hi) = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });

    if lo.is_finite() && hi.is_finite() {
        (lo, hi)
    } else {
        (0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_finite_min_max() {
        let data = vec![1.0, f32::NAN, 5.0, -2.0, f32::INFINITY, 3.0];
        assert_eq!(compute_finite_min_max(&data), (-2.0, 5.0));

        let empty: Vec<f32> = vec![];
        assert_eq!(compute_finite_min_max(&empty), (0.0, 1.0));

        let nans = vec![f32::NAN, f32::INFINITY];
        assert_eq!(compute_finite_min_max(&nans), (0.0, 1.0));
    }
}
