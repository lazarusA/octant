//! `spatial:` keys inherited from groups, kept together.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use zarrs::storage::ReadableStorageTraits;

use super::geozarr::link_pyramids;
use super::tests::var;
use crate::data::crs::inherit_zarr_group_keys;
use crate::data::ome::MetaReader;
use crate::data::ome::test_support::Fixture;
use crate::utils::metadata::merge_parent_attributes;

fn strings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn an_own_affine_takes_no_parent_grid_keys() {
    let parent = strings(&[
        ("spatial:shape", "[99, 99]"),
        ("spatial:registration", "node"),
        ("proj:code", "EPSG:4326"),
    ]);
    let mut own = strings(&[("spatial:transform", "[1, 0, 0, 0, -1, 0]")]);
    merge_parent_attributes(&mut own, &parent);
    assert!(!own.contains_key("spatial:shape") && !own.contains_key("spatial:registration"));
    assert_eq!(own.get("proj:code").map(String::as_str), Some("EPSG:4326"));
    let mut bare = HashMap::new();
    merge_parent_attributes(&mut bare, &parent);
    assert!(
        bare.contains_key("spatial:shape"),
        "no own affine: the group's keys"
    );
}

#[test]
fn zone_arrays_take_their_groups_affine() {
    // A geoemb `utm_zones` group: the affine on the zone group, no x/y arrays.
    let zone = json!({
        "spatial:transform": [10.0, 0.0, 0.0, 0.0, -10.0, 100.0],
        "spatial:dimensions": ["y", "x"],
        "spatial:shape": [10, 20],
    });
    let store = Fixture::default()
        .v3_group("", json!({}))
        .v3_group("utm31", zone)
        .memory();
    let storage: Arc<dyn ReadableStorageTraits> = store;
    let reader = MetaReader::direct(storage);
    let mut vars = vec![
        var("utm31/embeddings", &[3, 10, 20], &["time", "y", "x"]),
        var("utm31/preview", &[3, 5, 10], &["time", "y", "x"]),
    ];
    inherit_zarr_group_keys(&reader, &mut vars);
    link_pyramids(&reader, &mut vars);
    let x = vars[0].metadata_coordinates().expect("placed");
    assert_eq!(x[0].1.first_number(), Some(5.0));
    assert_eq!(vars[1].affine, None, "not on the zone's grid");
}

#[test]
fn levels_without_an_affine_take_no_inherited_one() {
    // The group's affine is level 0's; level 1 gives only a `transform`.
    let affine = json!([1.0, 0.0, 0.0, 0.0, -1.0, 8.0]);
    let coarse = json!({ "asset": "1", "derived_from": "0",
        "transform": { "scale": [2.0, 2.0] } });
    let group = json!({
        "spatial:transform": affine,
        "multiscales": { "layout": [{ "asset": "0" }, coarse] }
    });
    let store = Fixture::default().v3_group("", group).memory();
    let storage: Arc<dyn ReadableStorageTraits> = store;
    let reader = MetaReader::direct(storage);
    let inherited = strings(&[("spatial:transform", "[1, 0, 0, 0, -1, 8]")]);
    let mut vars = vec![
        var("0/b", &[8, 8], &["y", "x"]),
        var("1/b", &[4, 4], &["y", "x"]),
    ];
    for v in &mut vars {
        v.attributes = inherited.clone();
    }
    link_pyramids(&reader, &mut vars);
    assert!(vars[0].affine.is_some(), "level 0 takes the group's");
    assert_eq!(vars[1].affine, None, "level 1 has none of its own");
}
