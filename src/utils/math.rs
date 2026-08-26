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

/// Linearly interpolates between two 3D points `a` and `b` by factor `t`.
#[inline]
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Standard cubic ease-in-out curve for smooth procedural transitions.
#[inline]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Fast non-cryptographic PRNG (xorshift64) returning a pseudo-random `f32` in `[0.0, 1.0)`.
pub fn xorshift64_f32(seed: &mut u64) -> f32 {
    let mut x = *seed;
    if x == 0 {
        x = 0x853c49e6748fea9b;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    ((x & 0x00ff_ffff) as f32) / (0x0100_0000 as f32)
}

/// Infers 3D volume/grid depth dimension from total elements count and 2D grid dimensions.
#[inline]
pub fn calculate_3d_depth(total_len: usize, width: u32, height: u32) -> u32 {
    (total_len as u32 / (width.max(1) * height.max(1))).max(1)
}

/// Applies cursor-centered zoom scaling and relative panning offset.
#[inline]
pub fn apply_zoom_pan_at_point(
    old_zoom: f32,
    old_pan: eframe::egui::Vec2,
    mouse_pos: eframe::egui::Pos2,
    center: eframe::egui::Pos2,
    scroll: f32,
    min_zoom: f32,
    max_zoom: f32,
) -> (f32, eframe::egui::Vec2) {
    let zoom_factor = (1.0 + scroll * 0.002).clamp(0.8, 1.25);
    let new_zoom = (old_zoom * zoom_factor).clamp(min_zoom, max_zoom);
    let zoom_ratio = new_zoom / old_zoom;
    let new_pan = old_pan * zoom_ratio + (mouse_pos - center) * (1.0 - zoom_ratio);
    (new_zoom, new_pan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_3d_depth() {
        assert_eq!(calculate_3d_depth(64 * 64 * 16, 64, 64), 16);
        assert_eq!(calculate_3d_depth(0, 64, 64), 1);
        assert_eq!(calculate_3d_depth(100, 0, 0), 100);
    }

    #[test]
    fn test_apply_zoom_pan_at_point() {
        let center = eframe::egui::pos2(500.0, 500.0);
        let mouse = eframe::egui::pos2(500.0, 500.0);
        let (zoom, pan) =
            apply_zoom_pan_at_point(1.0, eframe::egui::Vec2::ZERO, mouse, center, 0.0, 0.1, 50.0);
        assert_eq!(zoom, 1.0);
        assert_eq!(pan, eframe::egui::Vec2::ZERO);
    }

    #[test]
    fn test_compute_finite_min_max() {
        let data = vec![1.0, f32::NAN, 5.0, -2.0, f32::INFINITY, 3.0];
        assert_eq!(compute_finite_min_max(&data), (-2.0, 5.0));

        let empty: Vec<f32> = vec![];
        assert_eq!(compute_finite_min_max(&empty), (0.0, 1.0));

        let nans = vec![f32::NAN, f32::INFINITY];
        assert_eq!(compute_finite_min_max(&nans), (0.0, 1.0));
    }

    #[test]
    fn test_lerp3() {
        let a = [0.0, 10.0, -5.0];
        let b = [10.0, 20.0, 5.0];
        assert_eq!(lerp3(a, b, 0.0), a);
        assert_eq!(lerp3(a, b, 1.0), b);
        assert_eq!(lerp3(a, b, 0.5), [5.0, 15.0, 0.0]);
    }

    #[test]
    fn test_ease_in_out_cubic() {
        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);
        assert_eq!(ease_in_out_cubic(0.5), 0.5);
        assert!(ease_in_out_cubic(0.25) < 0.25); // slow start
        assert!(ease_in_out_cubic(0.75) > 0.75); // fast middle, decelerating end
    }

    #[test]
    fn test_xorshift64_f32() {
        let mut seed = 0x123456789abcdef0;
        let v1 = xorshift64_f32(&mut seed);
        let v2 = xorshift64_f32(&mut seed);
        assert!((0.0..1.0).contains(&v1));
        assert!((0.0..1.0).contains(&v2));
        assert_ne!(v1, v2);

        // Seed 0 fallback
        let mut zero_seed = 0;
        let vz = xorshift64_f32(&mut zero_seed);
        assert!((0.0..1.0).contains(&vz));
        assert_ne!(zero_seed, 0);
    }
}
