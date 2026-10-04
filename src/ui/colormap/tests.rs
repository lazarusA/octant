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
