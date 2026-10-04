//! Unit tests for color scaling, colormaps, and color evaluation.

use super::*;
use crate::plots::common::PlotColorParams;

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
fn test_linear_scale_identity() {
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
fn test_log_scale_round_trip() {
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
fn test_symlog_scale() {
    let min_val = 0.0;
    let max_val = 1000.0;
    let c = 1.0;

    for &val in &[0.0, 10.0, 100.0, 500.0, 1000.0] {
        let pos = apply_color_scale_cpu(val, min_val, max_val, 2, c);
        let restored = unscale_norm_to_value(pos, min_val, max_val, 2, c);
        assert_close(restored, val, 1e-3);
    }
}

#[test]
fn test_evaluate_color_cpu() {
    let mut params = PlotColorParams {
        colormap: 0,
        cmin: 0.0,
        cmax: 100.0,
        use_nan_color: 1,
        use_lowclip: 1,
        use_highclip: 1,
        scale_type: 0,
        scale_param: 1.0,
        is_categorical: 0,
        num_categories: 10,
        nan_color: [1.0, 0.0, 0.0, 1.0],
        lowclip_color: [0.0, 1.0, 0.0, 1.0],
        highclip_color: [0.0, 0.0, 1.0, 1.0],
        ..Default::default()
    };

    // NaN color
    let nan_c = evaluate_color_cpu(f32::NAN, &params);
    assert_eq!(nan_c, egui::Color32::from_rgb(255, 0, 0));

    // Lowclip color
    let low_c = evaluate_color_cpu(-10.0, &params);
    assert_eq!(low_c, egui::Color32::from_rgb(0, 255, 0));

    // Highclip color
    let high_c = evaluate_color_cpu(150.0, &params);
    assert_eq!(high_c, egui::Color32::from_rgb(0, 0, 255));

    // Normal viridis midpoint
    let mid_c = evaluate_color_cpu(50.0, &params);
    assert_eq!(mid_c, registry::sample(0, 0.5));

    // RGB composite mode
    params.colormap = COLORMAP_RGB_COMPOSITE;
    let packed_rgb = (100u32) | (150u32 << 8) | (200u32 << 16);
    let rgb_c = evaluate_color_cpu(packed_rgb as f32, &params);
    assert_eq!(rgb_c, egui::Color32::from_rgb(100, 150, 200));

    // Categorical mode
    params.colormap = 0;
    params.is_categorical = 1;
    params.num_categories = 4;
    // 0..25 should map to first bin center (0.5/4 = 0.125)
    let cat_c = evaluate_color_cpu(10.0, &params);
    assert_eq!(cat_c, registry::sample(0, 0.125));

    // Unclipped values outside the range take the colormap ends, unquantized (as the GPU)
    params.use_lowclip = 0;
    params.use_highclip = 0;
    assert_eq!(evaluate_color_cpu(-10.0, &params), registry::sample(0, 0.0));
    assert_eq!(evaluate_color_cpu(150.0, &params), registry::sample(0, 1.0));
}

#[test]
fn lut_sampling_clamps_infinities_and_maps_nan_to_the_start() {
    let id = registry::default_id();
    assert_eq!(registry::sample(id, f32::NAN), registry::sample(id, 0.0));
    assert_eq!(
        registry::sample(id, f32::INFINITY),
        registry::sample(id, 1.0)
    );
    assert_eq!(
        registry::sample(id, f32::NEG_INFINITY),
        registry::sample(id, 0.0)
    );
}

#[test]
fn unknown_rows_sample_the_default_colormap() {
    let _registry = registry::test_lock();
    let unknown = u32::try_from(registry::rows()).unwrap_or(u32::MAX);
    assert!(!registry::is_row(unknown));
    assert!(!registry::is_stepped(unknown));
    let default = registry::default_id();
    assert_eq!(
        registry::sample(unknown, 0.3),
        registry::sample(default, 0.3)
    );
}
