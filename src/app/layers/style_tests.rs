//! `ColorStyle`: shader color uniforms and bounds resets (registry rows read
//! under `registry::test_lock`).

use super::ColorStyle;
use crate::data::matrix_data::MatrixData;
use crate::utils::colormap::{COLORMAP_RGB_COMPOSITE, registry};

fn categories(values: Vec<f32>) -> MatrixData {
    MatrixData::new(values.len(), 1, values, 1.0, 3.0, "test".into(), 1)
}

#[test]
fn params_carry_the_range_clips_and_clamped_opacity() {
    let _registry = registry::test_lock();
    let style = ColorStyle {
        range_min: -2.0,
        range_max: 7.0,
        use_lowclip: true,
        opacity: 1.5,
        scale_type: 2,
        scale_param: 3.0,
        ..Default::default()
    };
    let p = style.params(registry::default_id(), true, false, None);
    assert_eq!((p.cmin, p.cmax), (-2.0, 7.0));
    assert_eq!((p.use_lowclip, p.use_highclip, p.use_nan_color), (1, 0, 0));
    assert_eq!((p.scale_type, p.scale_param), (2, 3.0));
    assert_eq!(p.opacity, 1.0, "opacity is clamped to [0, 1]");
    assert_eq!(p.reverse, 1);
    assert_eq!(p.lowclip_color, style.lowclip_color);
}

#[test]
fn composites_draw_rgb_and_keep_the_row_as_fallback() {
    let _registry = registry::test_lock();
    let row = registry::default_id();
    let p = ColorStyle::default().params(row, false, true, None);
    assert_eq!(p.colormap, COLORMAP_RGB_COMPOSITE);
    assert_eq!(p.fallback_colormap, row);
}

#[test]
fn a_row_outside_the_atlas_draws_the_default() {
    let _registry = registry::test_lock();
    let p = ColorStyle::default().params(u32::MAX - 1, false, false, None);
    assert_eq!(p.colormap, registry::default_id());
    assert_eq!(p.fallback_colormap, registry::default_id());
}

#[test]
fn categorical_colors_count_the_distinct_values() {
    let _registry = registry::test_lock();
    let matrix = categories(vec![1.0, 2.0, 2.0, 3.0]);
    let style = ColorStyle {
        categorical: true,
        ..Default::default()
    };
    let p = style.params(registry::default_id(), false, false, Some(&matrix));
    assert_eq!((p.is_categorical, p.num_categories), (1, 3));
    let p = style.params(registry::default_id(), false, false, None);
    assert_eq!((p.is_categorical, p.num_categories), (1, 10), "no data");
    let p = ColorStyle::default().params(registry::default_id(), false, false, Some(&matrix));
    assert_eq!((p.is_categorical, p.num_categories), (0, 10), "continuous");
}

#[test]
fn reset_bounds_keeps_a_locked_extent() {
    let mut style = ColorStyle {
        global_min: 1.0,
        global_max: 2.0,
        lock_bounds: true,
        ..Default::default()
    };
    style.reset_bounds();
    assert_eq!((style.global_min, style.global_max), (1.0, 2.0));
    style.lock_bounds = false;
    style.reset_bounds();
    assert_eq!((style.global_min, style.global_max), (f32::MAX, f32::MIN));
}
