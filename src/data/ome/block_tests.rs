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
