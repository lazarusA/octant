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

/// Computes the maximum steps along the animated dimension that fit within GPU limits.
pub fn calculate_max_animated_steps(
    shape: &[u64],
    active_dims: &[bool],
    selected_ranges: &[(usize, usize)],
    anim_dim: usize,
    max_gpu_elements: usize,
) -> (usize, usize, usize) {
    let rank = shape.len();
    if anim_dim >= rank {
        return (1, 1, 1);
    }

    let mut spatial_elements_per_step: usize = 1;
    for (d, &size) in shape.iter().enumerate() {
        if d == anim_dim {
            continue;
        }
        if active_dims.get(d).copied().unwrap_or(false) {
            let span = if let Some(&(start, end)) = selected_ranges.get(d) {
                end.saturating_sub(start) + 1
            } else {
                size as usize
            };
            spatial_elements_per_step = spatial_elements_per_step.saturating_mul(span.max(1));
        }
    }
    if spatial_elements_per_step == 0 {
        spatial_elements_per_step = 1;
    }

    let full_anim_size = shape[anim_dim] as usize;
    let max_allowed =
        (max_gpu_elements / spatial_elements_per_step).clamp(1, full_anim_size.max(1));

    let requested = if let Some(&(start, end)) = selected_ranges.get(anim_dim) {
        end.saturating_sub(start) + 1
    } else {
        full_anim_size
    };

    (max_allowed, requested, spatial_elements_per_step)
}

/// Calculates requested download bytes and total file size for a variable.
pub fn calculate_download_sizes(
    shape: &[u64],
    file_size: u64,
    dtype_bytes: u64,
    active_dims: &[bool],
    selected_ranges: &[(usize, usize)],
) -> (u64, u64) {
    let total_elements: u64 = shape
        .iter()
        .copied()
        .try_fold(1u64, |acc, x| acc.checked_mul(x))
        .unwrap_or(u64::MAX)
        .max(1);
    let total_bytes = if file_size > 0 {
        file_size
    } else {
        total_elements.saturating_mul(dtype_bytes)
    };

    let mut requested_elements: u64 = 1;
    for (i, &size) in shape.iter().enumerate() {
        let dim_size = size as usize;
        if active_dims.get(i).copied().unwrap_or(false) {
            let span = if let Some(&(start, end)) = selected_ranges.get(i) {
                (end.saturating_sub(start) + 1).min(dim_size)
            } else {
                dim_size
            };
            requested_elements = requested_elements.saturating_mul(span.max(1) as u64);
        }
    }
    let requested_bytes = requested_elements.saturating_mul(dtype_bytes);

    (requested_bytes, total_bytes)
}

/// Calculates the total 3D volume elements from active dimensions.
pub fn calculate_volume_elements(
    shape: &[u64],
    active_dims: &[bool],
    selected_ranges: &[(usize, usize)],
) -> usize {
    let mut total_elements = 1usize;
    let mut counted = 0;
    for (i, &size) in shape.iter().enumerate() {
        let is_active = active_dims.get(i).copied().unwrap_or(false);
        if is_active {
            let (start, end) = selected_ranges
                .get(i)
                .copied()
                .unwrap_or((0, (size as usize).saturating_sub(1)));
            let span = (end.saturating_sub(start) + 1).min(size as usize);
            total_elements = total_elements.saturating_mul(span.max(1));
            counted += 1;
        }
    }
    if counted == 0 {
        shape
            .iter()
            .copied()
            .try_fold(1usize, |acc, x| acc.checked_mul(x as usize))
            .unwrap_or(usize::MAX)
    } else {
        total_elements
    }
}

/// Calculates the total 2D plane elements for spatial X and Y dimensions.
pub fn calculate_2d_elements(
    shape: &[u64],
    x_dim: usize,
    y_dim: usize,
    selected_ranges: &[(usize, usize)],
) -> usize {
    let rank = shape.len();
    let get_span = |d: usize| -> usize {
        if d >= rank {
            return 1;
        }
        let size = shape[d] as usize;
        if let Some(&(start, end)) = selected_ranges.get(d) {
            (end.saturating_sub(start) + 1).min(size)
        } else {
            size
        }
    };

    let nx = get_span(x_dim);
    let ny = if rank <= 1 || x_dim == y_dim {
        1
    } else {
        get_span(y_dim)
    };
    nx.saturating_mul(ny)
}

/// Computes normalized surface height on the 3D surface mesh matching surface.wgsl.
#[inline]
pub fn compute_normalized_surface_height(
    val: f32,
    cmin: f32,
    cmax: f32,
    surface_mode: u32,
    disp: f32,
) -> f32 {
    if !val.is_finite() {
        return 0.0;
    }
    let range = (cmax - cmin).max(1e-6);
    let mult = match surface_mode {
        1 => 0.6, // Flat Steps
        _ => 0.8, // Smooth Terrain (0) and 3D Lego Cubes (2)
    };

    if cmin < 0.0 && cmax > 0.0 {
        let max_abs = cmin.abs().max(cmax.abs());
        (val / max_abs).clamp(-1.0, 1.0) * mult * disp
    } else {
        let norm_val = ((val - cmin) / range).clamp(0.0, 1.0);
        norm_val * mult * disp
    }
}

/// Formats tick values cleanly using integer/decimal or concise scientific notation.
pub fn format_scientific_tick(val: f32) -> String {
    ScientificTick(val).to_string()
}

/// A tick value formatted like `format_scientific_tick`, written straight
/// into a formatter so per-frame UI code can use a stack buffer
/// (`stack_str(&mut buf, format_args!("{}", ScientificTick(v)))`).
#[derive(Clone, Copy)]
pub struct ScientificTick(pub f32);

/// Room for a formatted `ScientificTick` (the longest, e.g. `-1.23e-45`, is 9 bytes).
pub const TICK_BUF: usize = 16;

impl std::fmt::Display for ScientificTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.0;
        let abs_val = val.abs();
        if abs_val == 0.0 {
            f.write_str("0")
        } else if !(0.001..10000.0).contains(&abs_val) {
            let mut buf = [0u8; 32];
            let s = crate::utils::stack_str(&mut buf, format_args!("{val:.2e}"));
            match s.split_once('e') {
                Some((mantissa, exponent)) => {
                    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
                    write!(f, "{mantissa}e{exponent}")
                }
                None => f.write_str(s),
            }
        } else if val.fract().abs() < 1e-5 {
            write!(f, "{val:.0}")
        } else if (val * 10.0).fract().abs() < 1e-5 {
            write!(f, "{val:.1}")
        } else {
            write!(f, "{val:.2}")
        }
    }
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

    #[test]
    fn test_compute_normalized_surface_height() {
        let h_mid = compute_normalized_surface_height(50.0, 0.0, 100.0, 0, 1.0);
        assert!((h_mid - 0.4).abs() < 1e-4);

        let h_nan = compute_normalized_surface_height(f32::NAN, 0.0, 100.0, 0, 1.0);
        assert_eq!(h_nan, 0.0);
    }

    #[test]
    fn test_format_scientific_tick() {
        assert_eq!(format_scientific_tick(0.0), "0");
        assert_eq!(format_scientific_tick(15.0), "15");
        assert_eq!(format_scientific_tick(0.000045), "4.5e-5");
    }

    #[test]
    fn test_scientific_tick_fits_its_stack_buffer() {
        for val in [
            -1.23e-45,
            -f32::MAX,
            f32::MIN_POSITIVE,
            -9999.99,
            f32::NAN,
            f32::NEG_INFINITY,
        ] {
            let mut buf = [0u8; TICK_BUF];
            let text = crate::utils::stack_str(&mut buf, format_args!("{}", ScientificTick(val)));
            assert_eq!(text, format_scientific_tick(val));
        }
    }

    #[test]
    fn test_calculate_volume_and_2d_elements() {
        let shape = vec![10, 32, 64];
        let active = vec![true, true, true];
        let ranges = vec![(0, 9), (0, 31), (0, 63)];
        assert_eq!(
            calculate_volume_elements(&shape, &active, &ranges),
            10 * 32 * 64
        );
        assert_eq!(calculate_2d_elements(&shape, 2, 1, &ranges), 64 * 32);
    }
}
