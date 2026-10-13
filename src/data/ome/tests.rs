//! Discovery of OME-Zarr images, labels, series, plates and wells.

use serde_json::json;

use super::axes::AxisKind;
use super::model::ChannelFamily;
use super::test_support::{Fixture, czyx_axes, multiscale_v04};

fn idr_like_v04() -> Fixture {
    let ms = multiscale_v04(
        czyx_axes(),
        &[vec![1.0, 0.5, 0.36, 0.36], vec![1.0, 0.5, 0.72, 0.72]],
    );
    let attrs = json!({
        "_creator": { "name": "omero-zarr" },
        "multiscales": [ms],
        "omero": {
            "channels": [
                { "label": "LaminB1", "color": "0000FF", "active": true, "inverted": false,
                  "family": "linear", "coefficient": 1.0,
                  "window": { "min": 0, "max": 65535, "start": 0, "end": 1500 } },
                { "label": 7, "color": "#FFFF00", "active": false, "inverted": true,
                  "family": "logarithmic", "coefficient": 2.0,
                  "window": { "min": 0, "max": 65535, "start": 10, "end": 900 } }
            ],
            "rdefs": { "defaultT": 0, "defaultZ": 118, "model": "greyscale" }
        }
    });
    Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[2, 236, 275, 271])
        .v2_array("1", &[2, 236, 138, 136])
}

#[test]
fn v04_image_levels_become_variables() {
    let found = idr_like_v04().discover().expect("ome");
    assert_eq!(found.dataset.version, "0.4");
    let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["0", "1"]);
    let v = &found.variables[1];
    assert_eq!(v.dimension_names, ["c", "z", "y", "x"]);
    assert_eq!(v.shape, [2, 236, 138, 136]);
    let ome = v.ome.as_ref().expect("ome ref");
    assert_eq!(ome.level, 1);
    assert_eq!(ome.scale(2), Some(f64::from(0.72f32)));
    let image = &ome.image;
    assert_eq!(image.axes[0].kind, AxisKind::Channel);
    assert_eq!(image.axes[3].unit_symbol(), Some("µm"));
    assert_eq!(image.z_axis(), Some(1));
    assert_eq!((image.default_t, image.default_z), (Some(0), Some(118)));
    assert!(image.greyscale);
}

#[test]
fn omero_channels_keep_every_rendering_setting() {
    let found = idr_like_v04().discover().expect("ome");
    let channels = &found.variables[0].ome.as_ref().expect("ome").image.channels;
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].label, "LaminB1");
    assert_eq!(channels[0].color, [0, 0, 255]);
    let second = &channels[1];
    assert_eq!(second.label, "7");
    assert_eq!(second.color, [255, 255, 0]);
    assert!(!second.active && second.inverted);
    assert_eq!(second.family, ChannelFamily::Logarithmic);
    assert_eq!(second.coefficient, 2.0);
    let w = second.window.expect("window");
    assert_eq!((w.min, w.max, w.start, w.end), (0.0, 65535.0, 10.0, 900.0));
}

#[test]
fn conforming_metadata_has_no_warnings() {
    let found = idr_like_v04().discover().expect("ome");
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
}

#[test]
fn unknown_keys_and_missing_versions_are_repaired_with_warnings() {
    let mut ms = multiscale_v04(czyx_axes(), &[vec![1.0, 1.0, 1.0, 1.0]]);
    ms["extra"] = json!(1);
    ms.as_object_mut().expect("obj").remove("version");
    let found = Fixture::default()
        .v2_group("", json!({ "multiscales": [ms] }))
        .v2_array("0", &[1, 2, 3, 4])
        .discover()
        .expect("ome");
    assert_eq!(found.variables.len(), 1);
    assert!(
        found.warnings.iter().any(|w| w.contains("extra")),
        "{:?}",
        found.warnings
    );
}

#[test]
fn translations_compose_with_multiscale_transforms() {
    let ms = json!({
        "version": "0.4",
        "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
        "datasets": [{ "path": "0", "coordinateTransformations": [
            { "type": "scale", "scale": [2.0, 2.0] },
            { "type": "translation", "translation": [1.0, 3.0] }
        ]}],
        "coordinateTransformations": [{ "type": "scale", "scale": [10.0, 1.0] }]
    });
    let found = Fixture::default()
        .v2_group("", json!({ "multiscales": [ms] }))
        .v2_array("0", &[4, 4])
        .discover()
        .expect("ome");
    let level = found.variables[0]
        .ome
        .as_ref()
        .expect("ome")
        .level()
        .expect("level")
        .clone();
    assert_eq!(level.scale, [20.0, 2.0]);
    assert_eq!(level.translation, [10.0, 3.0]);
}

fn with_labels(f: Fixture) -> Fixture {
    let label_ms = multiscale_v04(czyx_axes(), &[vec![1.0, 0.5, 0.36, 0.36]]);
    f.v2_group("labels", json!({ "labels": ["cells"] }))
        .v2_group(
            "labels/cells",
            json!({
                "multiscales": [label_ms],
                "image-label": {
                    "version": "0.4",
                    "colors": [
                        { "label-value": 9, "rgba": [9, 9, 9, 255] },
                        { "label-value": 2, "rgba": [2, 2, 2, 128] }
                    ],
                    "properties": [{ "label-value": 2, "class": "nucleus", "area": 31 }],
                    "source": { "image": "../../" }
                }
            }),
        )
        .v2_array("labels/cells/0", &[1, 236, 275, 271])
}

#[test]
fn label_images_are_found_under_their_image() {
    let found = with_labels(idr_like_v04()).discover().expect("ome");
    let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["0", "1", "labels/cells/0"]);
    let image = &found.variables[0].ome.as_ref().expect("ome").image;
    assert_eq!(image.labels, ["cells"]);
    let label = &found.variables[2].ome.as_ref().expect("ome").image;
    let info = label.label_info.as_ref().expect("label info");
    assert_eq!(info.colors[0], (2, [2, 2, 2, 128]));
    assert_eq!(info.color_of(9), Some([9, 9, 9, 255]));
    let props = info.properties_of(2).expect("properties");
    assert!(props.contains(&("class".to_string(), "nucleus".to_string())));
    assert!(props.contains(&("area".to_string(), "31".to_string())));
    assert_eq!(info.source_image.as_deref(), Some("../../"));
}

fn v05_image() -> Fixture {
    let ome = json!({
        "version": "0.5",
        "multiscales": [{
            "name": "nuclei",
            "axes": [{ "name": "y", "type": "space", "unit": "nanometer" }, { "name": "x", "type": "space", "unit": "nanometer" }],
            "datasets": [{ "path": "s0", "coordinateTransformations": [{ "type": "scale", "scale": [4.0, 4.0] }] }]
        }]
    });
    Fixture::default()
        .v3_group("", json!({ "ome": ome }))
        .v3_array("s0", &[8, 8], &["y", "x"])
}

#[test]
fn v05_namespaced_image() {
    let found = v05_image().discover().expect("ome");
    assert_eq!(found.dataset.version, "0.5");
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    let v = &found.variables[0];
    assert_eq!(v.name, "s0");
    assert_eq!(v.long_name.as_deref(), Some("nuclei"));
    assert_eq!(
        v.ome.as_ref().expect("ome").image.axes[0].unit_symbol(),
        Some("nm")
    );
}

#[test]
fn web_passes_find_the_same_variables() {
    for fixture in [with_labels(idr_like_v04()), v05_image()] {
        let direct = fixture.discover().expect("direct");
        let (web, passes) = fixture.discover_prefetched();
        let web = web.expect("web");
        let names = |d: &super::Discovery| {
            d.variables
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&direct), names(&web));
        assert!(passes <= 6, "{passes} passes");
    }
}

#[test]
fn plain_zarr_is_not_ome() {
    let f = Fixture::default()
        .v2_group("", json!({ "title": "x" }))
        .v2_array("a", &[2]);
    assert!(f.discover().is_none());
    assert!(f.discover_prefetched().0.is_none());
}

#[test]
fn misspelled_axis_units_keep_the_multiscale() {
    let axes = json!([
        { "name": "y", "type": "space", "units": "micrometer" },
        { "name": "x", "type": "space", "units": "micrometer", "note": "x" }
    ]);
    let found = Fixture::default()
        .v2_group(
            "",
            json!({ "multiscales": [multiscale_v04(axes, &[vec![1.0, 1.0]])] }),
        )
        .v2_array("0", &[2, 2])
        .discover()
        .expect("ome");
    assert_eq!(found.variables.len(), 1);
    let image = &found.variables[0].ome.as_ref().expect("ome").image;
    assert_eq!(image.axes[1].unit_symbol(), Some("µm"));
    assert!(found.warnings.iter().any(|w| w.contains("units")));
}

#[test]
fn other_conventions_with_ome_like_keys_stay_plain_zarr() {
    let land_cover = Fixture::default()
        .v2_group(
            "",
            json!({ "labels": ["forest", "water"], "title": "land cover", "source": "x" }),
        )
        .v2_array("cover", &[4, 4]);
    assert!(land_cover.discover().is_none());
    let omero_only = Fixture::default()
        .v2_group("", json!({ "omero": { "channels": [] } }))
        .v2_array("a", &[2]);
    assert!(omero_only.discover().is_none());
    assert!(omero_only.discover_prefetched().0.is_none());
}
