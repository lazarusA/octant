//! Tests for the bundled catalog, LUT sampling, the registry and custom colormaps.

use super::*;
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::{format, lut};

#[test]
fn bundled_catalog_decodes_with_licensed_families() {
    let catalog = builtin();
    assert!(
        catalog.maps.len() > 200,
        "catalog has {} maps",
        catalog.maps.len()
    );
    for family in &catalog.families {
        assert!(!family.license.is_empty(), "{} has no license", family.key);
        assert!(
            !family.attribution.is_empty(),
            "{} has no attribution",
            family.key
        );
    }
    let mut keys: Vec<&str> = catalog.maps.iter().map(|m| m.key.as_str()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(
        keys.len(),
        catalog.maps.len(),
        "colormap keys must be unique"
    );
}

#[test]
fn bundled_licenses_cover_every_family() {
    let licenses = LICENSES_TEXT;
    for family in &builtin().families {
        assert!(
            licenses.contains(&format!("## {} (", family.name)),
            "LICENSES.md lacks a section for {}",
            family.name
        );
    }
}

#[test]
fn previously_shipped_colormaps_remain_available() {
    for key in [
        "matplotlib:viridis",
        "matplotlib:plasma",
        "matplotlib:inferno",
        "matplotlib:magma",
        "classic:turbo",
        "classic:coolwarm",
        "classic:cividis",
    ] {
        assert!(registry::find(key).is_some(), "missing {key}");
    }
    assert_eq!(
        registry::key_of(registry::default_id()),
        registry::DEFAULT_COLORMAP_KEY
    );
}

#[test]
fn viridis_matches_reference_endpoints() {
    let id = registry::default_id();
    assert_eq!(
        sample_colormap_rgb(id, 0.0),
        egui::Color32::from_rgb(68, 1, 84)
    );
    assert_eq!(
        sample_colormap_rgb(id, 1.0),
        egui::Color32::from_rgb(253, 231, 37)
    );
}

#[test]
fn lut_resampling_and_sampling() {
    let lut = lut::resample(&[[0, 0, 0], [255, 255, 255]], false);
    assert_eq!(lut[0], [0, 0, 0, 255]);
    assert_eq!(lut[255], [255, 255, 255, 255]);
    assert_eq!(
        sample_lut(&lut, 0.5),
        egui::Color32::from_rgb(128, 128, 128)
    );
    assert_eq!(sample_lut(&lut, f32::NAN), egui::Color32::from_rgb(0, 0, 0));

    let steps = lut::resample(&[[255, 0, 0], [0, 0, 255]], true);
    assert_eq!(steps[127], [255, 0, 0, 255]);
    assert_eq!(steps[128], [0, 0, 255, 255]);
    assert_eq!(orient(0.25, true), 0.75);
}

#[test]
fn reversed_params_flip_sampling() {
    let mut params = PlotColorParams {
        colormap: registry::default_id(),
        reverse: 1,
        ..Default::default()
    };
    params.cmin = 0.0;
    params.cmax = 1.0;
    let id = params.colormap;
    assert_eq!(
        evaluate_color_cpu(0.0, &params),
        sample_colormap_rgb(id, 1.0)
    );
    assert_eq!(
        evaluate_color_cpu(1.0, &params),
        sample_colormap_rgb(id, 0.0)
    );
}

#[test]
fn format_round_trips() {
    let records = format::CatalogRecords {
        families: vec![format::FamilyRecord {
            key: "f".into(),
            name: "F".into(),
            license: "MIT".into(),
            source: "s".into(),
            attribution: "a".into(),
        }],
        maps: vec![format::MapRecord {
            name: "m".into(),
            family: 0,
            kind: ColormapKind::Cyclic,
            stops: vec![[1, 2, 3], [4, 5, 6]],
        }],
    };
    let bytes = format::encode(&records).unwrap_or_default();
    assert_eq!(format::decode(&bytes), Ok(records));
    assert!(format::decode(&bytes[..bytes.len() - 1]).is_err());
    assert!(format::decode(b"nope").is_err());
}

#[test]
fn custom_colormaps_register_replace_and_remove() {
    let spec = CustomColormapSpec {
        name: "test_custom_roundtrip".into(),
        colors: "black, white".into(),
        interpolation: Interpolation::Linear,
        blend: BlendSpace::Rgb,
        classes: 0,
    };
    let Ok(entry) = spec.to_entry() else {
        panic!("valid spec rejected")
    };
    assert_eq!(entry.lut[0], [0, 0, 0, 255]);
    assert_eq!(entry.lut[255], [255, 255, 255, 255]);

    let generation = registry::generation();
    let id = registry::upsert_custom(entry);
    assert!(registry::generation() > generation);
    assert_eq!(registry::find(&spec.key()), Some(id));

    let red = CustomColormapSpec {
        colors: "red, red".into(),
        ..spec.clone()
    };
    let Ok(red_entry) = red.to_entry() else {
        panic!("valid spec rejected")
    };
    assert_eq!(
        registry::upsert_custom(red_entry),
        id,
        "same name replaces in place"
    );
    assert_eq!(
        sample_colormap_rgb(id, 0.5),
        egui::Color32::from_rgb(255, 0, 0)
    );

    assert!(registry::remove_custom(&spec.key()));
    assert_eq!(registry::find(&spec.key()), None);
}

#[test]
fn custom_colormap_validation() {
    let unnamed = CustomColormapSpec {
        name: "  ".into(),
        ..Default::default()
    };
    assert!(unnamed.to_entry().is_err());
    let bad = CustomColormapSpec {
        name: "x".into(),
        colors: "notacolor, blue".into(),
        ..Default::default()
    };
    assert!(bad.to_entry().is_err());
    let classes = CustomColormapSpec {
        name: "x".into(),
        colors: "red, blue".into(),
        classes: 2,
        ..Default::default()
    };
    let Ok(entry) = classes.to_entry() else {
        panic!("valid spec rejected")
    };
    assert_eq!(entry.kind, ColormapKind::Categorical);
    assert_eq!(entry.lut[0], [255, 0, 0, 255]);
    assert_eq!(entry.lut[255], [0, 0, 255, 255]);
}

#[test]
fn short_categorical_palettes_get_smooth_twins() {
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
    assert_eq!(
        sample_colormap_rgb(twin, 0.0),
        sample_colormap_rgb(set1, 0.0)
    );
    assert_eq!(
        sample_colormap_rgb(twin, 1.0),
        sample_colormap_rgb(set1, 1.0)
    );
    let distinct = |id: u32| {
        let mut seen: Vec<egui::Color32> = (0..=255)
            .map(|i| sample_colormap_rgb(id, i as f32 / 255.0))
            .collect();
        seen.dedup();
        seen.len()
    };
    assert!(distinct(twin) > 100, "smooth twin is a gradient");
    assert!(distinct(set1) < 40, "original stays stepped");
}

#[test]
fn smooth_rows_follow_the_atlas_order() {
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
