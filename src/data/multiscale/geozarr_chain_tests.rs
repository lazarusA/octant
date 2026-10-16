//! GeoZarr `derived_from` chains and affines: cycles and node registration.

use serde_json::json;

use super::geozarr_tests::{assert_halving, entry, link, pyramid_vars, root};

#[test]
fn derived_from_cycles_fall_back_to_the_shapes() {
    let entries = json!([
        { "asset": "0", "derived_from": "2", "transform": { "scale": [1.0, 2.0, 2.0] } },
        { "asset": "1", "derived_from": "0", "transform": { "scale": [1.0, 2.0, 2.0] } },
        { "asset": "2", "derived_from": "1", "transform": { "scale": [1.0, 2.0, 2.0] } }
    ]);
    let mut vars = pyramid_vars();
    link(json!({ "multiscales": { "layout": entries } }), &mut vars);
    // The chain loops: factors come from the level shapes instead.
    assert_halving(&vars);
}

#[test]
fn node_registered_affines_locate_cell_centres() {
    // Each level's affine names its first cell's centre: the corners align.
    let entries = (0..3)
        .map(|l| {
            let cell = (1u64 << l) as f64;
            let mut e = entry(l, cell, 0.0);
            e["spatial:transform"] = json!([cell, 0.0, cell / 2.0, 0.0, -cell, 32.0 - cell / 2.0]);
            e
        })
        .collect();
    let mut attrs = root(entries);
    attrs["spatial:registration"] = json!("node");
    let mut vars = pyramid_vars();
    link(attrs, &mut vars);
    assert_halving(&vars);
}
