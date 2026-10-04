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
        registry::sample(id, 0.0),
        egui::Color32::from_rgb(68, 1, 84)
    );
    assert_eq!(
        registry::sample(id, 1.0),
        egui::Color32::from_rgb(253, 231, 37)
    );
}

#[test]
fn lut_resampling_and_sampling() {
    let lut = lut::resample(&[[0, 0, 0], [255, 255, 255]], false);
    assert_eq!(lut[0], [0, 0, 0, 255]);
    assert_eq!(lut[255], [255, 255, 255, 255]);
    assert_eq!(
        lut::sample_lut(&lut, 0.5, false),
        egui::Color32::from_rgb(128, 128, 128)
    );
    assert_eq!(
        lut::sample_lut(&lut, f32::NAN, false),
        egui::Color32::from_rgb(0, 0, 0)
    );

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
    assert_eq!(evaluate_color_cpu(0.0, &params), registry::sample(id, 1.0));
    assert_eq!(evaluate_color_cpu(1.0, &params), registry::sample(id, 0.0));
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

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(format::decode(&trailing).is_err());
    // The kind byte precedes the stop count (u16) and the two RGB stops.
    let mut bad_kind = bytes;
    let kind_at = bad_kind.len() - 6 - 2 - 1;
    bad_kind[kind_at] = 99;
    assert!(format::decode(&bad_kind).is_err());
}

#[test]
fn custom_colormaps_register_replace_and_remove() {
    let _registry = crate::utils::colormap::registry::test_lock();
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
        registry::sample(id, 0.5),
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
