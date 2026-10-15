//! GeoZarr pyramids (zarr-conventions `multiscales` v0.1) read as stacks.

use std::sync::Arc;

use serde_json::{Value, json};
use zarrs::storage::ReadableStorageTraits;

use super::geozarr::link_pyramids;
use super::geozarr_layout::Layout;
use super::tests::var;
use super::types::MultiscaleKind;
use crate::data::metadata::VariableInfo;
use crate::data::ome::MetaReader;
use crate::data::ome::test_support::Fixture;

/// A level entry as topozarr writes it: `scale` relative to `derived_from`.
fn entry(level: usize, cell: f64, translation: f64) -> Value {
    let size = 32 / (1u64 << level);
    let affine = json!([cell, 0.0, -0.5, 0.0, -cell, 31.5]);
    if level == 0 {
        return json!({ "asset": "0", "spatial:transform": affine, "spatial:shape": [size, size] });
    }
    json!({
        "asset": level.to_string(), "derived_from": (level - 1).to_string(),
        "transform": { "scale": [1.0, 2.0, 2.0], "translation": [0.0, translation, translation] },
        "resampling_method": "mean", "spatial:transform": affine, "spatial:shape": [size, size]
    })
}

fn root(entries: Vec<Value>) -> Value {
    json!({
        "zarr_conventions": [{ "name": "multiscales" }, { "name": "spatial:" }],
        "spatial:dimensions": ["y", "x"],
        "multiscales": { "layout": entries, "resampling_method": "mean" }
    })
}

fn pyramid_vars() -> Vec<VariableInfo> {
    let mut vars = Vec::new();
    for l in 0..3u64 {
        let n = 32 >> l;
        vars.push(var(&format!("{l}/t2m"), &[2, n, n], &["time", "y", "x"]));
        vars.push(var(&format!("{l}/sst"), &[2, n, n], &["time", "y", "x"]));
    }
    vars
}

fn link(attrs: Value, vars: &mut [VariableInfo]) {
    let store = Fixture::default().v3_group("", attrs).memory();
    let storage: Arc<dyn ReadableStorageTraits> = store;
    link_pyramids(&MetaReader::direct(storage), vars);
}

fn assert_halving(vars: &[VariableInfo]) {
    for (i, v) in vars.iter().enumerate() {
        let r = v.multiscale.as_ref().expect("linked");
        assert_eq!((r.level, r.set.kind), (i / 2, MultiscaleKind::GeoZarr));
        assert_eq!(r.set.name, v.leaf_name());
        let lvl = r.info().expect("level");
        let f = (1u64 << r.level) as f64;
        assert_eq!(lvl.factor, [1.0, f, f], "{}", v.name);
        assert_eq!(lvl.offset, [0.0, 0.0, 0.0], "{}", v.name);
    }
}

#[test]
fn topozarr_pyramids_stack_every_variable() {
    let mut vars = pyramid_vars();
    link(
        root((0..3).map(|l| entry(l, (1u64 << l) as f64, 0.0)).collect()),
        &mut vars,
    );
    assert_halving(&vars);
    let set = &vars[0].multiscale.as_ref().expect("linked").set;
    assert_eq!(set.to_level(2, 2, 0..32), Some(0..8));
    assert_eq!(set.to_level(2, 0, 1..2), Some(1..2));
}

#[test]
fn cell_centre_translations_defer_to_the_affines() {
    // topozarr 0.1.9 wrote `(step - 1) / 2`; the affines keep corners aligned.
    let mut vars = pyramid_vars();
    link(
        root((0..3).map(|l| entry(l, (1u64 << l) as f64, 0.5)).collect()),
        &mut vars,
    );
    assert_halving(&vars);
}

#[test]
fn chained_transforms_without_affines() {
    let entries = json!([
        { "asset": "0" },
        { "asset": "1", "derived_from": "0", "transform": { "scale": [1.0, 2.0, 2.0] } },
        { "asset": "2", "derived_from": "1", "transform": { "scale": [1.0, 2.0, 2.0] } }
    ]);
    let mut vars = pyramid_vars();
    link(json!({ "multiscales": { "layout": entries } }), &mut vars);
    assert_halving(&vars);
}

#[test]
fn layout_order_does_not_matter() {
    let mut entries: Vec<Value> = (0..3).map(|l| entry(l, (1u64 << l) as f64, 0.0)).collect();
    entries.reverse();
    let mut vars = pyramid_vars();
    link(root(entries), &mut vars);
    assert_halving(&vars);
}

#[test]
fn array_assets_take_the_group_name() {
    let entries = json!([
        { "asset": "0" },
        { "asset": "1", "derived_from": "0", "transform": { "scale": [2.0, 2.0] } }
    ]);
    let store = Fixture::default()
        .v3_group("", json!({}))
        .v3_group("dem", json!({ "multiscales": { "layout": entries } }))
        .memory();
    let mut vars = vec![
        var("dem/0", &[64, 64], &["y", "x"]),
        var("dem/1", &[32, 32], &["y", "x"]),
    ];
    let storage: Arc<dyn ReadableStorageTraits> = store;
    link_pyramids(&MetaReader::direct(storage), &mut vars);
    let r = vars[1].multiscale.as_ref().expect("linked");
    assert_eq!((r.set.name.as_str(), r.level), ("dem", 1));
    assert_eq!(r.info().expect("level").factor, [2.0, 2.0]);
}

#[test]
fn ome_multiscales_and_plain_groups_are_not_layouts() {
    let ome = json!({ "multiscales": [{ "version": "0.4", "datasets": [{ "path": "0" }] }] });
    for attrs in [ome, json!({}), json!({ "multiscales": { "layout": [] } })] {
        let Value::Object(map) = attrs else {
            unreachable!()
        };
        assert_eq!(Layout::parse(&map), None);
    }
}

#[test]
fn levels_of_another_rank_stay_apart() {
    let mut vars = pyramid_vars();
    vars[4] = var("2/t2m", &[16, 16], &["y", "x"]);
    link(
        root((0..3).map(|l| entry(l, (1u64 << l) as f64, 0.0)).collect()),
        &mut vars,
    );
    assert!(vars[4].multiscale.is_none());
    let r = vars[0].multiscale.as_ref().expect("linked");
    assert_eq!(r.set.len(), 2);
}
