//! Composite hover readout: band combination names and per-channel raw values.

use std::collections::HashMap;

use super::composite::{CompositeKind, classify_rgb, composite_fields, composite_kind};
use super::entries_2d::resolve_2d_plot_entries;
use super::field::HoverField;
use crate::app::{OctantApp, StoreKind};
use crate::data::octant_block::OctantBlock;
use crate::data::{DatasetMetadata, VariableInfo};

const BANDS: [&str; 4] = ["Blue", "Green", "Red", "NIR"];
const SHAPE: [usize; 3] = [4, 2, 3];

/// Four bands of 2x3 pixels whose raw value encodes `band * 100 + y * 10 + x`.
fn block() -> OctantBlock {
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
        HashMap::new(),
    )
}

fn metadata(labels: Option<&[&str]>) -> DatasetMetadata {
    let dimension_coordinates = labels
        .map(|l| {
            HashMap::from([(
                "band".to_string(),
                l.iter().map(|s| s.to_string()).collect(),
            )])
        })
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

fn kind(app: &OctantApp) -> CompositeKind {
    let meta = app.plotted_dataset_metadata.as_ref();
    composite_kind(app, meta, meta.and_then(|m| m.variables.first()))
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
    let meta = app.plotted_dataset_metadata.as_ref();
    let var = meta.and_then(|m| m.variables.first());
    assert_eq!(
        composite_fields(&app, meta, var, "", (2, 1)),
        [
            HoverField::new("R", "NIR 312"),
            HoverField::new("G", "Red 212"),
            HoverField::new("B", "Green 112"),
        ]
    );
    let unnamed = composite_app(None, [3, 2, 1]);
    let meta = unnamed.plotted_dataset_metadata.as_ref();
    let var = meta.and_then(|m| m.variables.first());
    let fields = composite_fields(&unnamed, meta, var, "m", (0, 0));
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
    let raw = |band: usize| band * 100 + py * 10 + px;
    let labels: Vec<_> = fields.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(labels, ["R", "G", "B", "y", "x"]);
    assert_eq!(fields[0].value, format!("NIR {}", raw(3)));
    assert_eq!(fields[2].value, format!("Green {}", raw(1)));
}
