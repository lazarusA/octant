//! Composite hover readout: band combination names and per-channel raw values.

use std::collections::HashMap;

use super::composite::{
    CompositeKind, CompositeLabels, build_labels, classify_rgb, composite_fields,
};
use super::entries_2d::resolve_2d_plot_entries;
use super::field::HoverField;
use crate::app::{OctantApp, StoreKind};
use crate::data::octant_block::OctantBlock;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

const BANDS: [&str; 4] = ["Blue", "Green", "Red", "NIR"];
const SHAPE: [usize; 3] = [4, 2, 3];

/// Four bands of 2x3 pixels whose raw value encodes `band * 100 + y * 10 + x`.
fn block() -> OctantBlock {
    block_with(HashMap::new())
}

fn block_with(attributes: HashMap<String, String>) -> OctantBlock {
    let values: Vec<f32> = (0..4)
        .flat_map(|b| (0..2).flat_map(move |y| (0..3).map(move |x| (b * 100 + y * 10 + x) as f32)))
        .collect();
    let dims = ["band", "y", "x"].map(String::from).to_vec();
    OctantBlock::new(
        "raster".into(),
        SHAPE.to_vec(),
        dims,
        vec![0, 0, 0],
        values,
        HashMap::new(),
        attributes,
    )
}

fn metadata(labels: Option<&[&str]>) -> DatasetMetadata {
    let dimension_coordinates = labels
        .and_then(|l| CoordValues::from_labels(l.iter().map(|s| s.to_string()).collect()))
        .map(|band| HashMap::from([("band".to_string(), band)]))
        .unwrap_or_default();
    DatasetMetadata {
        name: "bands.tif".into(),
        store_type: "GeoTIFF".into(),
        variables: vec![VariableInfo {
            name: "raster".into(),
            shape: SHAPE.map(|n| n as u64).to_vec(),
            dimension_names: ["band", "y", "x"].map(String::from).to_vec(),
            ..Default::default()
        }],
        dimension_coordinates,
    }
}

/// An app showing `block()` as an RGB composite of `channels`.
fn composite_app(labels: Option<&[&str]>, channels: [usize; 3]) -> OctantApp {
    let mut app = OctantApp {
        plotted_dataset_metadata: Some(metadata(labels)),
        plotted_store_kind: StoreKind::LocalGeoTiff,
        rgb_composite_mode: true,
        rgb_composite_channels: channels,
        ..Default::default()
    };
    app.apply_2d_projection(&block(), 2, 1, (0, 3), (0, 2), &[0, 0, 0], true, 0);
    app
}

fn labels(app: &OctantApp) -> CompositeLabels {
    let meta = app.plotted_dataset_metadata.as_ref();
    build_labels(app, meta, meta.and_then(|m| m.variables.first()))
}

fn kind(app: &OctantApp) -> CompositeKind {
    labels(app).kind
}

#[test]
fn band_names_decide_true_and_false_color() {
    assert_eq!(
        classify_rgb([Some("Red"), Some(" green "), Some("BLUE")]),
        CompositeKind::TrueColor
    );
    assert_eq!(
        classify_rgb([Some("NIR"), Some("Red"), Some("Green")]),
        CompositeKind::FalseColor
    );
    assert_eq!(
        classify_rgb([Some("Red"), None, Some("Blue")]),
        CompositeKind::Rgb
    );

    assert_eq!(
        kind(&composite_app(Some(&BANDS), [2, 1, 0])),
        CompositeKind::TrueColor
    );
    assert_eq!(
        kind(&composite_app(Some(&BANDS), [3, 2, 1])),
        CompositeKind::FalseColor
    );
    assert_eq!(kind(&composite_app(None, [2, 1, 0])), CompositeKind::Rgb);
    // Generated placeholders are not names.
    let partial = ["Band 1", "Green", "Red", "Band 4"];
    assert_eq!(
        kind(&composite_app(Some(&partial), [3, 2, 1])),
        CompositeKind::Rgb
    );
}

#[test]
fn channel_rows_show_each_band_and_its_raw_value() {
    let app = composite_app(Some(&BANDS), [3, 2, 1]);
    assert_eq!(
        composite_fields(&app, &labels(&app), "", (2, 1)),
        [
            HoverField::new("R", "NIR 312"),
            HoverField::new("G", "Red 212"),
            HoverField::new("B", "Green 112"),
        ]
    );
    let unnamed = composite_app(None, [3, 2, 1]);
    let fields = composite_fields(&unnamed, &labels(&unnamed), "m", (0, 0));
    assert_eq!(fields[0], HoverField::new("R", "Band 4 300 m"));
}

#[test]
fn composite_hover_lists_channels_instead_of_a_band_row() {
    let app = composite_app(Some(&BANDS), [3, 2, 1]);
    let meta = app.plotted_dataset_metadata.as_ref();
    let var = meta.and_then(|m| m.variables.first());
    let matrix = app.matrix_data.as_ref().expect("composite matrix");
    let (val, fields, px, py) =
        resolve_2d_plot_entries(&app, matrix, meta, var, 2.5 / 3.0, 1.5 / 2.0, None);

    assert!(!val.is_nan(), "composite pixel holds a packed color");
    let names: Vec<_> = fields.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(
        names,
        ["y", "x"],
        "no band row: the composite mixes every band"
    );
    // The tooltip prepends one row per channel at the hovered pixel.
    let raw = |band: usize| band * 100 + py * 10 + px;
    let channels = composite_fields(&app, &labels(&app), "", (px, py));
    assert_eq!(channels[0], HoverField::new("R", format!("NIR {}", raw(3))));
    assert_eq!(
        channels[2],
        HoverField::new("B", format!("Green {}", raw(1)))
    );
}

#[test]
fn cmyk_composite_is_named_and_lists_its_inks() {
    let inks = ["Cyan", "Magenta", "Yellow", "Black"];
    let mut app = composite_app(Some(&inks), [0, 1, 2]);
    let cmyk = HashMap::from([("photometric".to_string(), "cmyk".to_string())]);
    app.apply_2d_projection(&block_with(cmyk), 2, 1, (0, 3), (0, 2), &[0, 0, 0], true, 0);
    assert_eq!(kind(&app), CompositeKind::Cmyk);

    assert_eq!(
        composite_fields(&app, &labels(&app), "", (1, 0)),
        [
            HoverField::new("C", "Cyan 1"),
            HoverField::new("M", "Magenta 101"),
            HoverField::new("Y", "Yellow 201"),
            HoverField::new("K", "Black 301"),
        ]
    );
}

#[test]
fn cmyk_volume_is_named_from_the_dataset_tags() {
    let mut meta = metadata(None);
    meta.variables[0]
        .attributes
        .insert("photometric".into(), "cmyk".into());
    let app = OctantApp {
        plotted_dataset_metadata: Some(meta),
        plotted_store_kind: StoreKind::LocalGeoTiff,
        rgb_composite_mode: true,
        ..Default::default()
    };
    assert!(app.composite_probe.is_none(), "volumes keep no probe");
    assert_eq!(kind(&app), CompositeKind::Cmyk);
}

#[test]
fn cmyk_volume_projection_draws_converted_inks() {
    let cmyk = HashMap::from([("photometric".to_string(), "cmyk".to_string())]);
    let block = block_with(cmyk);
    let mut app = OctantApp {
        plotted_dataset_metadata: Some(metadata(None)),
        plotted_store_kind: StoreKind::LocalGeoTiff,
        active_plot_type: crate::plots::PlotType::Volume,
        rgb_composite_mode: true,
        ..Default::default()
    };
    // The band axis is the channel, so the raster becomes a one-voxel-deep volume.
    let (req, local) = (((0, 2), (0, 1), (0, 0)), ((0, 3), (0, 2), (0, 1)));
    app.apply_3d_volume_projection(
        &block,
        2,
        1,
        0,
        req.0,
        req.1,
        req.2,
        local.0,
        local.1,
        local.2,
        &[0, 0, 0],
        false,
        true,
        0,
    );

    let expected = crate::data::slicing::slice_cmyk_composite(&block, 3, 2, 6, 1);
    let volume = app.volume_data.as_ref().expect("volume data");
    assert_eq!(
        volume.values,
        expected.expect("cmyk composite").values.to_vec()
    );
}
