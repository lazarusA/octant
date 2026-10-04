//! Picker filtering tests.

use super::search::{FamilyFilter, FilterKey, filter_ids};
use crate::utils::colormap::{ColormapKind, builtin, registry};

#[test]
fn empty_filter_lists_every_builtin() {
    let ids = filter_ids(&FilterKey::default());
    assert!(ids.len() >= builtin().maps.len());
}

#[test]
fn query_matches_name_case_insensitively() {
    let key = FilterKey {
        query: "VIRIDIS".into(),
        ..Default::default()
    };
    let ids = filter_ids(&key);
    let found = ids
        .iter()
        .any(|&id| registry::with_entry(id, |e| e.key == "matplotlib:viridis").unwrap_or(false));
    assert!(found);
}

#[test]
fn query_matches_family_name() {
    let key = FilterKey {
        query: "cmocean".into(),
        ..Default::default()
    };
    let ids = filter_ids(&key);
    assert!(ids.len() >= 20);
}

#[test]
fn kind_and_family_filters_compose() {
    let Some(cmocean) = builtin().families.iter().position(|f| f.key == "cmocean") else {
        panic!("cmocean family missing");
    };
    let key = FilterKey {
        kind: Some(ColormapKind::Diverging),
        family: Some(FamilyFilter::Builtin(cmocean)),
        ..Default::default()
    };
    for id in filter_ids(&key) {
        let ok = registry::with_entry(id, |e| {
            e.kind == ColormapKind::Diverging && e.family == Some(cmocean)
        });
        assert_eq!(ok, Some(true));
    }
    assert!(!filter_ids(&key).is_empty());
}

#[test]
fn long_names_are_elided_to_fit() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let font = egui::FontId::proportional(13.0);
        let color = egui::Color32::WHITE;
        let long = "linear_protanopic_deuteranopic_kbjyw_5_95_c25";
        let cut = super::label::elided(ui, long, font.clone(), color, 80.0);
        assert!(cut.elided);
        assert!(cut.size().x <= 80.5);
        let short = super::label::elided(ui, "fire", font, color, 80.0);
        assert!(!short.elided);
        assert_eq!(short.text(), "fire");
    });
    // Headless frame: font atlas updates are intentionally not uploaded.
    output.textures_delta.clear();
}
