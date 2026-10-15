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

/// The EOPF Sentinel-2 L2A `measurements/reflectance` layout (explorer
/// store, January 2026), trimmed to four levels: `scale` is written
/// relative to the finest level whatever `derived_from` names, translations
/// shift cell centres, and bands start at the level of their resolution.
fn eopf_reflectance() -> serde_json::Value {
    let level = |asset: &str, from: Option<&str>, step: f64, n: u64| {
        let mut entry = json!({
            "asset": asset,
            "spatial:shape": [n, n],
            "spatial:transform": [step, 0.0, 600000.0, 0.0, -step, 5600040.0]
        });
        if let Some(from) = from {
            let s = step / 10.0;
            entry["derived_from"] = from.into();
            entry["transform"] =
                json!({ "scale": [s, s], "translation": [(s - 1.0) * 5.0, (s - 1.0) * 5.0] });
        }
        entry
    };
    json!({
        "zarr_conventions": [{ "name": "multiscales" }, { "name": "spatial:" }, { "name": "proj:" }],
        "spatial:dimensions": ["y", "x"],
        "spatial:registration": "pixel",
        "proj:code": "EPSG:32636",
        "multiscales": {
            "layout": [
                level("r10m", None, 10.0, 10980),
                level("r20m", Some("r10m"), 20.0, 5490),
                level("r60m", Some("r10m"), 60.0, 1830),
                level("r120m", Some("r60m"), 120.0, 915)
            ],
            "resampling_method": "average"
        }
    })
}

#[test]
fn eopf_bands_stack_from_their_finest_level() {
    let yx = ["y", "x"];
    let mut vars = vec![
        var("r10m/b02", &[10980, 10980], &yx),
        var("r20m/b02", &[5490, 5490], &yx),
        var("r20m/b05", &[5490, 5490], &yx),
        var("r60m/b02", &[1830, 1830], &yx),
        var("r60m/b05", &[1830, 1830], &yx),
        var("r60m/b01", &[1830, 1830], &yx),
        var("r120m/b02", &[915, 915], &yx),
        var("r120m/b05", &[915, 915], &yx),
        var("r120m/b01", &[915, 915], &yx),
        var("r10m/spatial_ref", &[], &[]),
        var("r20m/spatial_ref", &[], &[]),
    ];
    link(eopf_reflectance(), &mut vars);
    let factors = |i: usize| {
        let r = vars[i].multiscale.as_ref().expect("linked");
        let set = &r.set;
        let f: Vec<f64> = set.levels.iter().map(|l| l.factor[1]).collect();
        (set.base().path.clone(), f)
    };
    // Cumulative `scale`s read through the affines: 120 m is 12x 10 m, not 72x.
    assert_eq!(factors(0), ("r10m/b02".into(), vec![1.0, 2.0, 6.0, 12.0]));
    assert_eq!(factors(2), ("r20m/b05".into(), vec![1.0, 3.0, 6.0]));
    assert_eq!(factors(5), ("r60m/b01".into(), vec![1.0, 2.0]));
    let offsets = &vars[6]
        .multiscale
        .as_ref()
        .expect("linked")
        .info()
        .expect("level")
        .offset;
    assert_eq!(offsets, &[0.0, 0.0], "the affines share their corner");
    assert!(vars[9].multiscale.is_none(), "scalars form no stack");
}
