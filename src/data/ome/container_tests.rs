//! Discovery of OME-Zarr containers: bioformats2raw series, plates and wells.

use serde_json::json;

use super::test_support::{Fixture, multiscale_v04};

fn bf2raw(listed: bool) -> Fixture {
    let ms = |name: &str| {
        json!({ "multiscales": [{
            "version": "0.4", "name": name,
            "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
            "datasets": [{ "path": "0", "coordinateTransformations": [{ "type": "scale", "scale": [1.0, 1.0] }] }]
        }]})
    };
    let mut root = json!({ "bioformats2raw.layout": 3 });
    if listed {
        root["series"] = json!(["0", "1"]);
    }
    let xml = br#"<?xml version="1.0"?><OME xmlns="http://www.openmicroscopy.org/Schemas/OME/2016-06">
        <Image ID="Image:0" Name="slide A"/><Image ID="Image:1" Name="slide B"><Pixels/></Image></OME>"#;
    Fixture::default()
        .v2_group("", root)
        .v2_group("0", ms(""))
        .v2_array("0/0", &[4, 4])
        .v2_group("1", ms("own name"))
        .v2_array("1/0", &[4, 4])
        .bytes("OME/METADATA.ome.xml", xml)
}

#[test]
fn bioformats2raw_series_are_probed_and_named() {
    for listed in [false, true] {
        let fixture = bf2raw(listed);
        for found in [
            fixture.discover().expect("ome"),
            fixture.discover_prefetched().0.expect("web"),
        ] {
            let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
            assert_eq!(names, ["0/0", "1/0"]);
            assert_eq!(found.dataset.group_name("0"), Some("slide A"));
            assert_eq!(found.dataset.group_name("1"), Some("slide B"));
            assert_eq!(found.variables[0].long_name.as_deref(), Some("slide A"));
            assert_eq!(found.variables[1].long_name.as_deref(), Some("own name"));
        }
    }
}

#[test]
fn series_listed_on_the_ome_group_are_not_probed() {
    let decoy = json!({ "multiscales": [{
        "version": "0.4",
        "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
        "datasets": [{ "path": "0", "coordinateTransformations": [{ "type": "scale", "scale": [1.0, 1.0] }] }]
    }]});
    let fixture = bf2raw(false)
        .v2_group("OME", json!({ "series": ["0", "1"] }))
        .v2_group("2", decoy)
        .v2_array("2/0", &[4, 4]);
    for found in [
        fixture.discover().expect("ome"),
        fixture.discover_prefetched().0.expect("web"),
    ] {
        let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["0/0", "1/0"]);
    }
}

#[test]
fn plates_list_wells_without_opening_them() {
    let plate = json!({ "plate": {
        "version": "0.4", "name": "screen",
        "rows": [{ "name": "A" }, { "name": "B" }], "columns": [{ "name": "1" }, { "name": "2" }],
        "wells": [{ "path": "A/1", "rowIndex": 0, "columnIndex": 0 }, { "path": "B/2", "rowIndex": 1, "columnIndex": 1 }],
        "acquisitions": [{ "id": 3, "name": "run" }], "field_count": 2
    }});
    let found = Fixture::default()
        .v2_group("", plate)
        .discover()
        .expect("ome");
    assert!(found.variables.is_empty());
    let plate = found.dataset.plate.expect("plate");
    assert_eq!(plate.rows, ["A", "B"]);
    let wells: Vec<_> = plate
        .wells
        .iter()
        .map(|w| (w.path.as_str(), w.row, w.column))
        .collect();
    assert_eq!(wells, [("A/1", 0, 0), ("B/2", 1, 1)]);
    assert_eq!(plate.acquisitions[0].id, 3);
}

#[test]
fn wells_open_their_fields() {
    let ms = multiscale_v04(
        json!([{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }]),
        &[vec![1.0, 1.0]],
    );
    let found = Fixture::default()
        .v2_group("", json!({ "well": { "version": "0.4", "images": [{ "path": "0", "acquisition": 1 }, { "path": "1" }] } }))
        .v2_group("0", json!({ "multiscales": [ms.clone()] }))
        .v2_array("0/0", &[2, 2])
        .v2_group("1", json!({ "multiscales": [ms] }))
        .v2_array("1/0", &[2, 2])
        .discover()
        .expect("ome");
    let names: Vec<&str> = found.variables.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["0/0", "1/0"]);
    let well = found.dataset.well.expect("well");
    assert_eq!(well.fields_of(Some(1)).count(), 2);
    assert_eq!(well.fields[0].acquisition, Some(1));
}
