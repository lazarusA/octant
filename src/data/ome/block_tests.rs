//! OME-Zarr level blocks: stored order, axis names and physical coordinates.

use serde_json::json;

use super::test_support::{Fixture, multiscale_v04};
use crate::data::blocks::BlockStore;
use crate::data::slice_request::{DimensionSelection, SliceRequest};

/// A 0.4 image `(y, x)` of 3 x 2 whose value is `10 * row + column`, with a
/// micrometer pixel size of 0.5 and an origin at (4, 2).
fn raster() -> Fixture {
    let ms = json!({
        "version": "0.4",
        "axes": [{ "name": "y", "type": "space", "unit": "micrometer" }, { "name": "x", "type": "space", "unit": "micrometer" }],
        "datasets": [{ "path": "0", "coordinateTransformations": [
            { "type": "scale", "scale": [0.5, 0.5] }, { "type": "translation", "translation": [4.0, 2.0] }
        ] }]
    });
    let values: Vec<u8> = [0u16, 1, 10, 11, 20, 21]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    Fixture::default()
        .v2_group("", json!({ "multiscales": [ms] }))
        .v2_array("0", &[3, 2])
        .bytes("0/0.0", &values)
}

#[test]
fn ome_blocks_keep_row_zero_on_top_and_take_axis_names() {
    let store =
        crate::data::backends::zarr::ZarrBlockStore::new(raster().memory(), "memory://ome_raster");
    let meta = store.inspect().expect("inspect");
    assert_eq!(meta.variables[0].dimension_names, ["y", "x"]);
    let request = SliceRequest::new(
        "0",
        vec![
            DimensionSelection::range(0, 3),
            DimensionSelection::range(0, 2),
        ],
    );
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(block.dimension_names, ["y", "x"]);
    assert!(block.flipped_dims.is_empty());
    assert_eq!(&block.values[..], &[0.0, 1.0, 10.0, 11.0, 20.0, 21.0]);
}

#[test]
fn ome_levels_get_physical_coordinates_scoped_to_their_level() {
    let store =
        crate::data::backends::zarr::ZarrBlockStore::new(raster().memory(), "memory://ome_coords");
    let meta = store.inspect().expect("inspect");
    let coords = store
        .variable_coordinates(&meta.variables[0])
        .expect("coords");
    let y = coords.get("0/y").expect("scoped y");
    assert_eq!(y.number(0), Some(4.0));
    assert_eq!(y.number(2), Some(5.0));
    assert_eq!(coords.get("0/x").and_then(|x| x.number(1)), Some(2.5));
}

#[test]
fn channel_labels_need_every_channel_named() {
    let axes = json!([{ "name": "c", "type": "channel" }, { "name": "x", "type": "space" }]);
    let ms = multiscale_v04(axes, &[vec![1.0, 1.0]]);
    let with = |labels: serde_json::Value| {
        let f = Fixture::default()
            .v2_group(
                "",
                json!({ "multiscales": [ms.clone()], "omero": { "channels": labels } }),
            )
            .v2_array("0", &[2, 4]);
        let store = crate::data::backends::zarr::ZarrBlockStore::new(f.memory(), "memory://ome_ch");
        let meta = store.inspect().expect("inspect");
        store
            .variable_coordinates(&meta.variables[0])
            .expect("coords")
    };
    let window = json!({ "min": 0, "max": 1, "start": 0, "end": 1 });
    let named = with(json!([
        { "label": "DAPI", "color": "0000FF", "window": window },
        { "label": "GFP", "color": "00FF00", "window": window }
    ]));
    assert_eq!(named.get("0/c").and_then(|c| c.label(1)), Some("GFP"));
    let partly = with(json!([
        { "label": "DAPI", "color": "0000FF", "window": window },
        { "color": "00FF00", "window": window }
    ]));
    assert!(!partly.contains_key("0/c"));
    // Index-only x (no unit, scale 1, no translation) gets no coordinate.
    assert!(!named.contains_key("0/x"));
}

#[test]
fn arrays_under_a_non_ome_multiscales_group_stay_plain_zarr() {
    // ndpyramid-style pyramid: a `multiscales` list without axes, levels
    // holding variables rather than arrays.
    let f = Fixture::default()
        .v2_group(
            "",
            json!({ "multiscales": [{ "datasets": [{ "path": "0" }] }] }),
        )
        .v2_group("0", json!({}))
        .v2_array("0/tas", &[2, 2])
        .json(
            "0/tas/.zattrs",
            json!({ "_ARRAY_DIMENSIONS": ["lat", "lon"] }),
        );
    let store = crate::data::backends::zarr::ZarrBlockStore::new(f.memory(), "memory://ndpyr");
    let request = SliceRequest::new(
        "0/tas",
        vec![
            DimensionSelection::range(0, 2),
            DimensionSelection::range(0, 2),
        ],
    );
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(block.dimension_names, ["lat", "lon"]);
}

#[test]
fn unreadable_image_label_reads_plain_values() {
    let ms = json!({
        "version": "0.4",
        "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
        "datasets": [{ "path": "0", "coordinateTransformations": [{ "type": "scale", "scale": [1.0, 1.0] }] }]
    });
    let values: Vec<u8> = [5u16, 0].iter().flat_map(|v| v.to_le_bytes()).collect();
    let f = Fixture::default()
        .v2_group(
            "",
            json!({ "multiscales": [ms], "image-label": { "colors": "not a list" } }),
        )
        .v2_array("0", &[1, 2])
        .bytes("0/0.0", &values);
    let store = crate::data::backends::zarr::ZarrBlockStore::new(f.memory(), "memory://badlabel");
    let meta = store.inspect().expect("inspect");
    assert!(
        meta.variables[0]
            .ome
            .as_ref()
            .is_some_and(|o| o.image.label_info.is_none())
    );
    let request = SliceRequest::new(
        "0",
        vec![
            DimensionSelection::range(0, 1),
            DimensionSelection::range(0, 2),
        ],
    );
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(&block.values[..], &[5.0, 0.0]);
}

#[test]
fn level_coordinates_never_fill_unscoped_keys() {
    let ms = json!({
        "version": "0.4",
        "axes": [{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }],
        "datasets": [
            { "path": "0", "coordinateTransformations": [{ "type": "scale", "scale": [1.0, 1.0] }] },
            { "path": "1", "coordinateTransformations": [{ "type": "scale", "scale": [2.0, 2.0] }] }
        ]
    });
    let f = Fixture::default()
        .v2_group("", json!({ "multiscales": [ms] }))
        .v2_array("0", &[4, 4])
        .v2_array("1", &[2, 2]);
    let store = crate::data::backends::zarr::ZarrBlockStore::new(f.memory(), "memory://ome_scoped");
    let meta = store.inspect().expect("inspect");
    let coords = store
        .variable_coordinates(&meta.variables[1])
        .expect("coords");
    assert!(coords.contains_key("1/x"));
    assert!(!coords.contains_key("x"));
}
