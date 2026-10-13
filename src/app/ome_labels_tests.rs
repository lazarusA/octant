//! OME-Zarr label images as overlays: exact ids, label colors and hover
//! readings with their properties.

use std::sync::Arc;

use serde_json::json;

use crate::app::layers::{Alignment, LayerId};
use crate::app::test_support::{TARGET, make_resident, selection_of};
use crate::app::{OctantApp, StoreKind};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::ome::test_support::Fixture;
use crate::data::{DataSource, Dataset, DatasetMetadata, SliceRequest, StoreHandle};
use crate::ui::hover::card::{HoverValue, LayerValue};
use crate::ui::hover::overlays::overlay_values;

/// Label ids past `f32`'s exact integers, and one past 2^31.
const IDS: [u32; 4] = [0, 7, 16_777_217, 3_000_000_000];

/// A `(y, x)` 2 x 2 image `0` and its label image `labels/cells/0`.
fn ome_app() -> (OctantApp, DatasetMetadata) {
    let axes = json!([{ "name": "y", "type": "space" }, { "name": "x", "type": "space" }]);
    let ms = json!({ "version": "0.4", "axes": axes, "datasets": [{ "path": "0",
        "coordinateTransformations": [{ "type": "scale", "scale": [1.0, 1.0] }] }] });
    let image: Vec<u8> = [1u16, 2, 3, 4]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let labels: Vec<u8> = IDS.iter().flat_map(|v| v.to_le_bytes()).collect();
    let fixture = Fixture::default()
        .v2_group("", json!({ "multiscales": [ms.clone()] }))
        .v2_array("0", &[2, 2])
        .bytes("0/0.0", &image)
        .v2_group("labels", json!({ "labels": ["cells"] }))
        .v2_group(
            "labels/cells",
            json!({ "multiscales": [ms], "image-label": { "version": "0.4",
                "colors": [{ "label-value": 7, "rgba": [255, 0, 0, 255] }],
                "properties": [{ "label-value": 7, "class": "nucleus" }] } }),
        )
        .v2_array_of("labels/cells/0", &[2, 2], "<u4")
        .bytes("labels/cells/0/0.0", &labels);
    let mut app = OctantApp::default();
    let zarr = GenericZarrBlockStore::new(fixture.memory(), TARGET, "zarr", "Zarr");
    let meta = zarr.inspect().expect("inspect");
    let source_id = StoreKind::make_source_id(StoreKind::RemoteZarr, TARGET);
    let source = DataSource::new(
        &source_id,
        StoreKind::RemoteZarr.to_data_source_kind(),
        TARGET,
        "Store",
    );
    let handle = StoreHandle::new(source.clone(), Arc::new(zarr));
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    for name in ["0", "labels/cells/0"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[2, 2]));
    }
    let base = selection_of(&mut app, &meta, "0");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    assert!(
        app.layers.base.data.matrix.is_some(),
        "the image is plotted"
    );
    (app, meta)
}

fn label_overlay(app: &mut OctantApp, meta: &DatasetMetadata) -> LayerId {
    let idx = meta
        .variables
        .iter()
        .position(|v| v.name == "labels/cells/0")
        .expect("label variable");
    app.add_overlay(idx).expect("overlay")
}

#[test]
fn label_images_overlay_their_image_with_exact_ids() {
    let (mut app, meta) = ome_app();
    let id = label_overlay(&mut app, &meta);
    let layer = app.layers.get(id).expect("layer");
    assert!(layer.labels().is_some(), "drawn as labels");
    assert_eq!(layer.alignment, Alignment::SameGrid);
    assert!(!layer.composite.enabled);
    let matrix = layer.data.matrix.as_ref().expect("label data");
    let ids: Vec<u32> = matrix.values.iter().map(|v| v.to_bits()).collect();
    assert_eq!(ids, IDS);
}

#[test]
fn hovering_a_label_reads_its_id_color_and_properties() {
    let (mut app, meta) = ome_app();
    label_overlay(&mut app, &meta);
    let mut rows = [LayerValue::EMPTY; crate::app::overlays::MAX_OVERLAYS];
    assert_eq!(overlay_values(&app, (1, 0), &mut rows), 1);
    assert_eq!(rows[0].value, HoverValue::Label(7));
    assert_eq!(rows[0].units, "class: nucleus");
    let opacity = app.layers.overlays()[0].color.opacity;
    let alpha = (255.0 * opacity).round() as u8;
    assert_eq!(
        rows[0].swatch,
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, alpha)
    );
    overlay_values(&app, (1, 1), &mut rows);
    assert_eq!(rows[0].value, HoverValue::Label(3_000_000_000));
    overlay_values(&app, (0, 0), &mut rows);
    assert_eq!(rows[0].value, HoverValue::NoData, "label 0 is background");
}

#[test]
fn label_colors_match_the_shader() {
    use crate::plots::labels::{label_color, palette_entries};
    let info = crate::data::ome::OmeLabelInfo {
        colors: vec![(7, [1, 2, 3, 4]), (1 << 40, [9, 9, 9, 9])],
        ..Default::default()
    };
    assert_eq!(
        palette_entries(&info),
        [[7, u32::from_le_bytes([1, 2, 3, 4])]]
    );
    assert_eq!(label_color(&info, 7), Some([1, 2, 3, 4]));
    assert_eq!(label_color(&info, 0), None);
    let hashed = label_color(&info, 8).expect("hashed");
    assert_eq!(hashed[3], 255);
    assert_eq!(label_color(&info, 8), Some(hashed), "stable");
}
