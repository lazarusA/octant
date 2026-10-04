//! Tests for smooth twins of categorical palettes and nearest-texel sampling
//! of stepped maps.

use super::*;

#[test]
fn short_categorical_palettes_get_smooth_twins() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let set1 = registry::find("colorbrewer:Set1").unwrap_or(u32::MAX);
    let viridis = registry::default_id();
    let glasbey = registry::find("colorcet:glasbey").unwrap_or(u32::MAX);
    let Some(twin) = registry::smooth_variant(set1) else {
        panic!("Set1 should have a smooth twin");
    };
    assert!(
        twin as usize >= registry::len(),
        "twins live after the colormap rows"
    );
    assert!((twin as usize) < registry::rows());
    assert_eq!(
        registry::smooth_variant(viridis),
        None,
        "continuous maps have no twin"
    );
    assert_eq!(
        registry::smooth_variant(glasbey),
        None,
        "long palettes are not smoothed"
    );

    // The twin keeps the palette's first and last colors and blends in between,
    // while the original stays a step function.
    assert_eq!(registry::sample(twin, 0.0), registry::sample(set1, 0.0));
    assert_eq!(registry::sample(twin, 1.0), registry::sample(set1, 1.0));
    let distinct = |id: u32| {
        let mut seen: Vec<egui::Color32> = (0..=255)
            .map(|i| registry::sample(id, i as f32 / 255.0))
            .collect();
        seen.dedup();
        seen.len()
    };
    assert!(distinct(twin) > 100, "smooth twin is a gradient");
    assert!(distinct(set1) < 40, "original stays stepped");
}

#[test]
fn smooth_rows_follow_the_atlas_order() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let mut rows = 0usize;
    registry::for_each_lut(|row, _| {
        assert_eq!(row as usize, rows);
        rows += 1;
    });
    assert_eq!(rows, registry::rows());
    assert!(registry::rows() > registry::len());
}

#[test]
fn custom_maps_with_classes_get_a_smooth_twin() {
    let stepped = CustomColormapSpec {
        name: "smooth_twin_test".into(),
        colors: "red, blue".into(),
        classes: 4,
        ..Default::default()
    };
    let Ok(entry) = stepped.to_entry() else {
        panic!("valid spec rejected")
    };
    let Some(smooth) = entry.smooth.as_deref() else {
        panic!("missing smooth twin")
    };
    assert_ne!(
        smooth[100], entry.lut[100],
        "twin blends where the stepped map is flat"
    );
    let continuous = CustomColormapSpec {
        classes: 0,
        ..stepped
    };
    let Ok(entry) = continuous.to_entry() else {
        panic!("valid spec rejected")
    };
    assert!(entry.smooth.is_none());
}

#[test]
fn stepped_maps_never_mix_two_palette_colors() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let tab10 = registry::find("classic:tab10").unwrap_or(u32::MAX);
    assert!(registry::is_stepped(tab10));
    let palette: Vec<egui::Color32> = (0..10)
        .map(|k| registry::sample(tab10, (k as f32 + 0.5) / 10.0))
        .collect();
    // Bin centers for any category count, including those landing between steps.
    for n in 1..=12 {
        for k in 0..n {
            let c = registry::sample(tab10, (k as f32 + 0.5) / n as f32);
            assert!(
                palette.contains(&c),
                "n={n} k={k} produced a mixed color {c:?}"
            );
        }
    }
    assert!(!registry::is_stepped(registry::default_id()));
    if let Some(twin) = registry::smooth_variant(tab10) {
        assert!(!registry::is_stepped(twin), "smooth twins blend");
    }
}
