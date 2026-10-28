//! How many decimals a coordinate needs.

/// Decimals of a coordinate when its spacing is unknown.
pub const COORD_DECIMALS: usize = 2;

/// Fewest decimals that tell values `step` apart (0.5 -> 1, 0.05 -> 2, 2 -> 0); 0 for
/// a zero or non-finite step.
pub fn step_decimals(step: f64) -> usize {
    let step = step.abs();
    if !(step.is_finite() && step > 0.0) {
        return 0;
    }
    (-step.log10() - 1e-9).ceil().max(0.0) as usize
}

/// Fewest decimals, at most `max`, that write `x` to within `tol` (179.875 -> 3, 0.25 -> 2);
/// `None` when more are needed (1/120) or `x` is not finite.
pub fn exact_decimals(x: f64, tol: f64, max: usize) -> Option<usize> {
    let mut scale = 1.0;
    for d in 0..=max {
        let scaled = x * scale;
        if (scaled - scaled.round()).abs() <= tol * scale {
            return Some(d);
        }
        scale *= 10.0;
    }
    None
}
