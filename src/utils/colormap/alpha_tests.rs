use super::alpha::{AlphaCurve, AlphaError, AlphaInterp, PRESETS, bake, parse};
use super::lut::{LUT_SIZE, sample_alpha};
use super::registry;

fn baked(text: &str, interp: AlphaInterp) -> Box<super::Lut> {
    let Ok(Some(curve)) = parse(text) else {
        panic!("{text} should parse");
    };
    bake(&curve, interp)
}

fn alpha_at(text: &str, interp: AlphaInterp, t: f32) -> f32 {
    sample_alpha(&baked(text, interp), t)
}

#[test]
fn parses_values_stops_and_blank() {
    assert_eq!(parse("  "), Ok(None));
    assert_eq!(
        parse("0.1, 0.4;0.3 0.1"),
        Ok(Some(AlphaCurve::Values(vec![0.1, 0.4, 0.3, 0.1])))
    );
    assert_eq!(
        parse("0:0, 0.5:1"),
        Ok(Some(AlphaCurve::Stops(vec![[0.0, 0.0], [0.5, 1.0]])))
    );
    assert_eq!(
        parse("0 : 0,  0.5 : 1 , 1: 0"),
        Ok(Some(AlphaCurve::Stops(vec![
            [0.0, 0.0],
            [0.5, 1.0],
            [1.0, 0.0]
        ])))
    );
}

#[test]
fn rejects_bad_input() {
    assert_eq!(parse("0.2, x"), Err(AlphaError::NotANumber("x".into())));
    assert_eq!(parse("1.5"), Err(AlphaError::OutOfRange(1.5)));
    assert_eq!(parse("0.2, 0:1"), Err(AlphaError::MixedForms));
    assert_eq!(parse("0.5:1, 0.2:0"), Err(AlphaError::Unsorted));
    for (label, text, _) in PRESETS {
        assert!(matches!(parse(text), Ok(Some(_))), "preset {label}");
    }
}

#[test]
fn linear_values_span_the_range() {
    let near = |a: f32, b: f32| (a - b).abs() < 1.0 / 255.0;
    assert!(near(alpha_at("0, 1", AlphaInterp::Linear, 0.0), 0.0));
    assert!(near(alpha_at("0, 1", AlphaInterp::Linear, 0.5), 0.5));
    assert!(near(alpha_at("0, 1", AlphaInterp::Linear, 1.0), 1.0));
    assert!(near(alpha_at("0.2, 1, 0.2", AlphaInterp::Linear, 0.5), 1.0));
    assert!(near(alpha_at("0.3", AlphaInterp::Linear, 0.7), 0.3));
}

#[test]
fn step_values_fill_equal_bins() {
    // Four values: one per quarter, matching four categorical bin centers.
    let lut = baked("0.1, 0.4, 0.3, 0.1", AlphaInterp::Step);
    for (bin, want) in [0.1_f32, 0.4, 0.3, 0.1].into_iter().enumerate() {
        let t = (bin as f32 + 0.5) / 4.0;
        assert!(
            (sample_alpha(&lut, t) - want).abs() < 1.0 / 255.0,
            "bin {bin}"
        );
    }
}

#[test]
fn step_values_match_many_category_centers() {
    // Alternating alphas expose any blending with a neighboring bin.
    for n in [64_usize, 100, 128] {
        let text = (0..n)
            .map(|b| if b % 2 == 0 { "0" } else { "1" })
            .collect::<Vec<_>>()
            .join(",");
        let lut = baked(&text, AlphaInterp::Step);
        for b in 0..n {
            let t = (b as f32 + 0.5) / n as f32;
            let want = (b % 2) as f32;
            let got = sample_alpha(&lut, t);
            assert!((got - want).abs() < 0.02, "{n} bins, bin {b}: {got}");
        }
    }
}

#[test]
fn stops_blend_or_hold() {
    let text = "0.2:0, 0.4:1, 0.4:0.5, 1:0.5";
    let near = |a: f32, b: f32| (a - b).abs() < 1.5 / 255.0;
    // Constant before the first stop, jump at repeated positions.
    assert!(near(alpha_at(text, AlphaInterp::Linear, 0.0), 0.0));
    assert!(near(alpha_at(text, AlphaInterp::Linear, 0.3), 0.5));
    assert!(near(alpha_at(text, AlphaInterp::Linear, 0.7), 0.5));
    assert!(near(
        alpha_at("0:1, 0.4:0, 0.6:1", AlphaInterp::Step, 0.5),
        0.0
    ));
    assert!(near(
        alpha_at("0:1, 0.4:0, 0.6:1", AlphaInterp::Step, 0.7),
        1.0
    ));
}

#[test]
fn full_lut_passes_through() {
    let text = (0..LUT_SIZE)
        .map(|i| format!("{}", i as f32 / 255.0))
        .collect::<Vec<_>>()
        .join(",");
    let lut = baked(&text, AlphaInterp::Linear);
    assert!(
        lut.iter()
            .enumerate()
            .all(|(i, px)| usize::from(px[3]) == i)
    );
}

#[test]
fn registry_holds_the_curve_as_last_row() {
    let _lock = registry::test_lock();
    let before = registry::rows();
    registry::set_alpha_curve(0, Some(baked("0, 1", AlphaInterp::Linear)));
    let row = registry::alpha_row(0);
    assert_eq!(registry::rows(), before + 1);
    assert_eq!(row, u32::try_from(before).ok());
    assert!(row.is_some_and(|r| !registry::is_row(r)));
    let mut last = None;
    registry::for_each_lut(|id, lut| last = Some((id, lut[0][3], lut[255][3])));
    assert_eq!(last, row.map(|r| (r, 0, 255)));
    assert!((registry::curve_alpha(0, 0.5) - 0.5).abs() < 1.0 / 255.0);
    registry::set_alpha_curve(0, None);
    assert_eq!(registry::rows(), before);
    assert_eq!(registry::alpha_row(0), None);
    assert_eq!(registry::curve_alpha(0, 0.5), 1.0);
}

#[test]
fn each_key_has_its_own_curve_row_in_key_order() {
    let _lock = registry::test_lock();
    let before = registry::rows();
    registry::set_alpha_curve(7, Some(baked("1, 0", AlphaInterp::Linear)));
    registry::set_alpha_curve(3, Some(baked("0, 1", AlphaInterp::Linear)));
    assert_eq!(registry::rows(), before + 2);
    assert_eq!(registry::alpha_row(3), u32::try_from(before).ok());
    assert_eq!(registry::alpha_row(7), u32::try_from(before + 1).ok());
    assert!(registry::curve_alpha(3, 0.9) > 0.8);
    assert!(registry::curve_alpha(7, 0.9) < 0.2);
    let Some(row) = registry::alpha_row(7) else {
        panic!("key 7 has a curve");
    };
    assert_eq!(
        registry::row_curve_alpha(row, 0.9),
        registry::curve_alpha(7, 0.9)
    );

    let generation = registry::generation();
    registry::set_alpha_curve(3, None);
    assert!(
        registry::generation() > generation,
        "removing a curve bumps the generation"
    );
    assert_eq!(registry::alpha_row(3), None);
    assert_eq!(
        registry::alpha_row(7),
        u32::try_from(before).ok(),
        "rows close up"
    );
    registry::set_alpha_curve(7, None);
    assert_eq!(registry::rows(), before);
}
