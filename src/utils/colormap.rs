use egui::Color32;

/// Samples colormap RGB color for normalized parameter t in [0.0, 1.0] matching WGSL shaders 100% bit-exactly.
pub fn sample_colormap_rgb(colormap_id: u32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let (r, g, b) = match colormap_id {
        0 => sample_viridis(t),
        1 => sample_plasma(t),
        2 => sample_inferno(t),
        3 => sample_magma(t),
        4 => sample_turbo(t),
        5 => sample_coolwarm(t),
        6 => sample_cividis(t),
        _ => sample_viridis(t),
    };

    Color32::from_rgb(
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Unscales a normalized colorbar position t in [0.0, 1.0] back to the exact data value v(t) for tick labeling.
pub fn unscale_norm_to_value(
    t: f32,
    cmin: f32,
    cmax: f32,
    scale_type: u32,
    scale_param: f32,
) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let range = (cmax - cmin).max(1e-30);

    match scale_type {
        1 => {
            if cmin < -1e-15 {
                return cmin + t * range;
            }
            let safe_min = if cmin <= 1e-15 {
                (cmax * 0.001).max(1e-12)
            } else {
                cmin
            };
            let safe_max = cmax.max(safe_min * 1.0001);
            let gamma = if scale_param > 0.0 && scale_param != 1.0 {
                scale_param
            } else {
                1.0
            };
            let t_gamma = t.powf(1.0 / gamma);

            let log_min = safe_min.ln();
            let log_max = safe_max.ln();
            let log_val = log_min + t_gamma * (log_max - log_min);
            log_val.exp()
        }
        2 => {
            let c = if scale_param > 0.0 { scale_param } else { 1.0 };
            let safe_range = range.abs().max(1e-6);
            let k = 1.0 + safe_range / c;
            let norm_x = (c * (k.powf(t) - 1.0)) / safe_range;
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        3 => {
            let y = 2.0 * t - 1.0;
            let norm_x = 0.5 + 0.5 * y.signum() * y.powi(2);
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        4 => {
            let k = if scale_param > 0.0 { scale_param } else { 3.0 };
            let denom = k.exp() - 1.0;
            let norm_x = if denom.abs() > 1e-5 {
                (1.0 + t * denom).ln() / k
            } else {
                t
            };
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        _ => cmin + t * range,
    }
}

/// Computes normalized colormap parameter t for raw value val under current scale.
pub fn apply_color_scale_cpu(
    val: f32,
    cmin: f32,
    cmax: f32,
    scale_type: u32,
    scale_param: f32,
) -> f32 {
    let range = (cmax - cmin).max(1e-30);

    match scale_type {
        1 => {
            if cmin < -1e-15 {
                return ((val - cmin) / range).clamp(0.0, 1.0);
            }
            let safe_min = if cmin <= 1e-15 {
                (cmax * 0.001).max(1e-12)
            } else {
                cmin
            };
            let safe_max = cmax.max(safe_min * 1.0001);
            if val <= safe_min {
                return 0.0;
            }
            let safe_v = val.clamp(safe_min, safe_max);
            let log_v = safe_v.ln();
            let log_min = safe_min.ln();
            let log_max = safe_max.ln();
            let log_range = (log_max - log_min).max(1e-6);
            let norm_log = ((log_v - log_min) / log_range).clamp(0.0, 1.0);
            let gamma = if scale_param > 0.0 && scale_param != 1.0 {
                scale_param
            } else {
                1.0
            };
            norm_log.powf(gamma)
        }
        2 => {
            let c = if scale_param > 0.0 { scale_param } else { 1.0 };
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let safe_range = range.abs().max(1e-6);
            let num = (c + norm_x * safe_range).ln() - c.ln();
            let denom = (c + safe_range).ln() - c.ln();
            if denom != 0.0 {
                (num / denom).clamp(0.0, 1.0)
            } else {
                norm_x
            }
        }
        3 => {
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let x_centered = 2.0 * norm_x - 1.0;
            (0.5 + 0.5 * x_centered.signum() * x_centered.abs().sqrt()).clamp(0.0, 1.0)
        }
        4 => {
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let k = if scale_param > 0.0 { scale_param } else { 3.0 };
            let num = (norm_x * k).exp() - 1.0;
            let denom = k.exp() - 1.0;
            if denom.abs() > 1e-5 {
                (num / denom).clamp(0.0, 1.0)
            } else {
                norm_x
            }
        }
        _ => ((val - cmin) / range).clamp(0.0, 1.0),
    }
}

fn mix_rgb(c0: [f32; 3], c1: [f32; 3], t: f32) -> (f32, f32, f32) {
    (
        c0[0] + (c1[0] - c0[0]) * t,
        c0[1] + (c1[1] - c0[1]) * t,
        c0[2] + (c1[2] - c0[2]) * t,
    )
}

fn sample_segmented(
    x: f32,
    c0: [f32; 3],
    c1: [f32; 3],
    c2: [f32; 3],
    c3: [f32; 3],
    c4: [f32; 3],
) -> (f32, f32, f32) {
    if x < 0.25 {
        mix_rgb(c0, c1, x / 0.25)
    } else if x < 0.50 {
        mix_rgb(c1, c2, (x - 0.25) / 0.25)
    } else if x < 0.75 {
        mix_rgb(c2, c3, (x - 0.50) / 0.25)
    } else {
        mix_rgb(c3, c4, (x - 0.75) / 0.25)
    }
}

// 0: Viridis (viridis.wgsl)
fn sample_viridis(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.267, 0.004, 0.329],
        [0.231, 0.322, 0.545],
        [0.129, 0.569, 0.551],
        [0.369, 0.788, 0.384],
        [0.992, 0.906, 0.145],
    )
}

// 1: Plasma (plasma.wgsl)
fn sample_plasma(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.051, 0.031, 0.529],
        [0.416, 0.000, 0.659],
        [0.694, 0.165, 0.565],
        [0.882, 0.392, 0.384],
        [0.941, 0.976, 0.129],
    )
}

// 2: Inferno (inferno.wgsl)
fn sample_inferno(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.000, 0.016],
        [0.341, 0.062, 0.429],
        [0.733, 0.216, 0.330],
        [0.976, 0.557, 0.035],
        [0.988, 1.000, 0.643],
    )
}

// 3: Magma (magma.wgsl)
#[allow(clippy::approx_constant)]
fn sample_magma(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.000, 0.016],
        [0.318, 0.071, 0.486],
        [0.714, 0.212, 0.475],
        [0.984, 0.533, 0.380],
        [0.988, 0.992, 0.749],
    )
}

// 4: Turbo (turbo.wgsl)
fn sample_turbo(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.190, 0.072, 0.232],
        [0.156, 0.447, 0.996],
        [0.134, 0.887, 0.525],
        [0.925, 0.875, 0.134],
        [0.900, 0.180, 0.090],
    )
}

// 5: Coolwarm (coolwarm.wgsl)
fn sample_coolwarm(t: f32) -> (f32, f32, f32) {
    let cool = [0.230, 0.299, 0.754];
    let mid = [0.865, 0.865, 0.865];
    let warm = [0.706, 0.016, 0.150];

    if t < 0.50 {
        mix_rgb(cool, mid, t / 0.50)
    } else {
        mix_rgb(mid, warm, (t - 0.50) / 0.50)
    }
}

// 6: Cividis (cividis.wgsl)
fn sample_cividis(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.135, 0.304],
        [0.286, 0.337, 0.435],
        [0.506, 0.514, 0.463],
        [0.741, 0.702, 0.430],
        [0.996, 0.906, 0.145],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f32, b: f32, tol: f32) {
        assert!(
            (a - b).abs() <= tol,
            "Expected {} to be close to {} within tol {}",
            a,
            b,
            tol
        );
    }

    #[test]
    fn test_1_linear_scale_identity() {
        let min_val = 0.0;
        let max_val = 100.0;
        for &norm_x in &[0.0, 0.25, 0.5, 1.0] {
            let val = min_val + norm_x * (max_val - min_val);
            let scaled = apply_color_scale_cpu(val, min_val, max_val, 0, 1.0);
            assert_close(scaled, norm_x, 1e-4);

            let unscaled = unscale_norm_to_value(scaled, min_val, max_val, 0, 1.0);
            assert_close(unscaled, val, 1e-4);
        }
    }

    #[test]
    fn test_2_log_scale_range_1_to_10() {
        let min_val: f32 = 1.0;
        let max_val: f32 = 10.0;
        let n = 10;
        let log_a = min_val.log10();
        let log_b = max_val.log10();

        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let log_val = log_a + t * (log_b - log_a);
            let raw_val = 10.0_f32.powf(log_val);

            let pos = apply_color_scale_cpu(raw_val, min_val, max_val, 1, 1.0);
            assert_close(pos, t, 1e-4);

            let restored = unscale_norm_to_value(pos, min_val, max_val, 1, 1.0);
            assert_close(restored, raw_val, 1e-4);
        }
    }

    #[test]
    fn test_2_log_scale_range_0001_to_1() {
        let min_val: f32 = 0.001;
        let max_val: f32 = 1.0;
        let n = 10;
        let log_a = min_val.log10();
        let log_b = max_val.log10();

        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let log_val = log_a + t * (log_b - log_a);
            let raw_val = 10.0_f32.powf(log_val);

            let pos = apply_color_scale_cpu(raw_val, min_val, max_val, 1, 1.0);
            assert_close(pos, t, 1e-4);

            let restored = unscale_norm_to_value(pos, min_val, max_val, 1, 1.0);
            assert_close(restored, raw_val, 1e-4);
        }
    }

    #[test]
    fn test_2_log_scale_zero_clipping() {
        let min_val = 0.0;
        let max_val = 1000.0;
        let data_range = max_val - min_val;

        assert_close(
            apply_color_scale_cpu(0.0, min_val, max_val, 1, 1.0),
            0.0,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(0.001 * data_range, min_val, max_val, 1, 1.0),
            0.0,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(0.01 * data_range, min_val, max_val, 1, 1.0),
            0.3333,
            1e-3,
        );
        assert_close(
            apply_color_scale_cpu(0.1 * data_range, min_val, max_val, 1, 1.0),
            0.6667,
            1e-3,
        );
        assert_close(
            apply_color_scale_cpu(1.0 * data_range, min_val, max_val, 1, 1.0),
            1.0,
            1e-4,
        );
    }

    #[test]
    fn test_3_symlog_offset_log_scale() {
        let min_val = 0.0;
        let max_val = 1000.0;
        let c = 1.0;

        assert_close(
            apply_color_scale_cpu(0.0, min_val, max_val, 2, c),
            0.0,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(10.0, min_val, max_val, 2, c),
            0.3472,
            1e-3,
        ); // norm_x = 0.01
        assert_close(
            apply_color_scale_cpu(100.0, min_val, max_val, 2, c),
            0.6680,
            1e-3,
        ); // norm_x = 0.1
        assert_close(
            apply_color_scale_cpu(1000.0, min_val, max_val, 2, c),
            1.0,
            1e-4,
        );

        // Round-trip test
        for &val in &[0.0, 10.0, 100.0, 500.0, 1000.0] {
            let pos = apply_color_scale_cpu(val, min_val, max_val, 2, c);
            let restored = unscale_norm_to_value(pos, min_val, max_val, 2, c);
            assert_close(restored, val, 1e-3);
        }
    }

    #[test]
    fn test_4_symmetric_sqrt_diverging() {
        let min_val = 0.0;
        let max_val = 100.0;

        assert_close(
            apply_color_scale_cpu(0.0, min_val, max_val, 3, 1.0),
            0.0,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(25.0, min_val, max_val, 3, 1.0),
            0.1464,
            1e-3,
        );
        assert_close(
            apply_color_scale_cpu(50.0, min_val, max_val, 3, 1.0),
            0.5,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(75.0, min_val, max_val, 3, 1.0),
            0.8536,
            1e-3,
        );
        assert_close(
            apply_color_scale_cpu(100.0, min_val, max_val, 3, 1.0),
            1.0,
            1e-4,
        );

        // Round-trip test
        for &val in &[0.0, 10.0, 25.0, 50.0, 75.0, 90.0, 100.0] {
            let pos = apply_color_scale_cpu(val, min_val, max_val, 3, 1.0);
            let restored = unscale_norm_to_value(pos, min_val, max_val, 3, 1.0);
            assert_close(restored, val, 1e-3);
        }
    }

    #[test]
    fn test_5_exponential_scale() {
        let min_val = 0.0;
        let max_val = 100.0;
        let k = 3.0;

        assert_close(
            apply_color_scale_cpu(0.0, min_val, max_val, 4, k),
            0.0,
            1e-4,
        );
        assert_close(
            apply_color_scale_cpu(100.0, min_val, max_val, 4, k),
            1.0,
            1e-4,
        );

        // Round-trip test
        for &val in &[0.0, 10.0, 25.0, 50.0, 75.0, 100.0] {
            let pos = apply_color_scale_cpu(val, min_val, max_val, 4, k);
            let restored = unscale_norm_to_value(pos, min_val, max_val, 4, k);
            assert_close(restored, val, 1e-3);
        }
    }
}
