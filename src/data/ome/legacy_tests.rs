//! Metadata from OME-NGFF 0.1-0.3, upgraded to 0.4.

use serde_json::json;

use super::axes::AxisKind;
use super::legacy::is_legacy;
use super::test_support::Fixture;

#[test]
fn legacy_versions() {
    assert!(is_legacy("0.1") && is_legacy("0.2") && is_legacy("0.3"));
    assert!(!is_legacy("0.4") && !is_legacy("0.5"));
}

#[test]
fn v01_without_axes_takes_tczyx_and_downsampling_scales() {
    let attrs = json!({
        "multiscales": [{ "version": "0.1", "datasets": [{ "path": "0" }, { "path": "1" }] }],
        "omero": { "channels": [{ "label": "Cam2-T1", "color": "FF0000", "window": { "start": 198, "end": 5090 } }] }
    });
    let found = Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[1, 4, 1, 1920, 1920])
        .v2_array("1", &[1, 4, 1, 960, 960])
        .discover()
        .expect("ome");
    assert_eq!(found.dataset.version, "0.1");
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    let v = &found.variables[1];
    assert_eq!(v.dimension_names, ["t", "c", "z", "y", "x"]);
    let ome = v.ome.as_ref().expect("ome");
    assert_eq!(ome.level().expect("level").scale, [1.0, 1.0, 1.0, 2.0, 2.0]);
    assert_eq!(ome.image.axes[1].kind, AxisKind::Channel);
    let w = ome.image.channels[0].window.expect("window");
    assert_eq!((w.min, w.max), (198.0, 5090.0));
}

#[test]
fn v03_string_axes_become_typed_axes() {
    let attrs = json!({
        "_creator": { "name": "omero-zarr", "version": "0.1.dev219" },
        "multiscales": [{ "version": "0.3", "axes": ["c", "z", "y", "x"],
            "datasets": [{ "path": "0" }, { "path": "1" }, { "path": "2" }] }],
        "omero": { "rdefs": { "defaultT": 0, "defaultZ": 118, "model": "color" },
            "channels": [{ "active": true, "color": "0000FF", "label": "LaminB1",
                "window": { "min": 0.0, "max": 65535.0, "start": 0.0, "end": 1500.0 } }] }
    });
    let found = Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[2, 236, 275, 271])
        .v2_array("1", &[2, 236, 138, 136])
        .v2_array("2", &[2, 236, 69, 68])
        .discover()
        .expect("ome");
    let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["0", "1", "2"]);
    let image = &found.variables[0].ome.as_ref().expect("ome").image;
    let kinds: Vec<AxisKind> = image.axes.iter().map(|a| a.kind).collect();
    assert_eq!(
        kinds,
        [
            AxisKind::Channel,
            AxisKind::Space,
            AxisKind::Space,
            AxisKind::Space
        ]
    );
    assert_eq!(image.default_z, Some(118));
    assert!(!image.greyscale);
    let scale = &image.levels[2].scale;
    assert_eq!(scale[0], 1.0);
    assert_eq!(scale[1], 1.0);
    assert!((scale[2] - 275.0 / 69.0).abs() < 1e-5);
}

#[test]
fn v04_missing_transforms_are_filled_with_a_warning() {
    let attrs = json!({ "multiscales": [{ "version": "0.4",
        "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
        "datasets": [{ "path": "0" }, { "path": "1" }] }] });
    let found = Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[8, 8])
        .v2_array("1", &[4, 4])
        .discover()
        .expect("ome");
    assert_eq!(found.variables.len(), 2);
    let scale = &found.variables[1].ome.as_ref().expect("ome").image.levels[1].scale;
    assert_eq!(scale, &[2.0, 2.0]);
    assert!(
        found
            .warnings
            .iter()
            .any(|w| w.contains("coordinateTransformations"))
    );
}
