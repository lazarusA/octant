//! Overlays, layer visibility, colors, the export dialog and plate wells.

use std::sync::Arc;

use super::{Done, Failure};
use crate::app::test_support::{TARGET, make_resident, memory_app};
use crate::app::{OctantApp, PlateState, StoreKind};
use crate::data::SliceRequest;
use crate::data::ome::OmePlate;
use crate::data::ome::plate::PlateWell;
use crate::export::ExportFormat;

/// `t2m` plotted, `sst` resident too.
fn plotted_app() -> OctantApp {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app.run_utterance("plot t2m");
    app
}

#[test]
fn an_overlay_is_added_hidden_colored_and_removed() {
    let mut app = plotted_app();
    let added = app.run_utterance("plot sst as an overlay");
    assert!(
        matches!(
            added.as_slice(),
            [Ok(Done::Selected(_)), Ok(Done::OverlayAdded(_))]
        ),
        "{added:?}"
    );
    assert_eq!(app.layers.overlays().len(), 1);
    let id = app.layers.overlays()[0].id();

    assert!(matches!(
        app.run_utterance("hide sst").as_slice(),
        [Ok(Done::LayerShown { visible: false, .. })]
    ));
    assert!(!app.layers.get(id).expect("overlay").visible);
    app.run_utterance("unhide sst");
    assert!(app.layers.get(id).expect("overlay").visible);

    app.run_utterance("colormap viridis for sst");
    let key = crate::utils::colormap::registry::key_of(
        app.layers.get(id).expect("overlay").color.colormap,
    );
    assert!(key.ends_with("viridis"), "the overlay's colormap: {key}");

    app.run_utterance("remove sst");
    assert!(app.layers.overlays().is_empty());
}

#[test]
fn no_overlay_is_added_without_a_plotted_heatmap() {
    let (mut app, meta) = memory_app();
    app.load_new_metadata(meta);
    let outcomes = app.run_utterance("plot sst as an overlay");
    assert!(
        matches!(outcomes.last(), Some(Err(Failure::OverlayUnavailable(_)))),
        "{outcomes:?}"
    );
}

#[test]
fn a_color_range_locks_the_base_layer_s_range() {
    let mut app = plotted_app();
    let outcomes = app.run_utterance("color range 250 to 300.5");
    assert!(
        matches!(outcomes.as_slice(), [Ok(Done::ColorRange { .. })]),
        "{outcomes:?}"
    );
    let color = &app.layers.base.color;
    assert_eq!(
        (color.range_min, color.range_max, color.lock_bounds),
        (250.0, 300.5, true)
    );
}

#[test]
fn export_opens_the_dialog_on_the_format_said() {
    let mut app = plotted_app();
    app.run_utterance("export as pdf");
    assert!(app.show_export_modal);
    assert_eq!(app.export_settings.format, ExportFormat::Pdf);
}

#[test]
fn a_well_is_picked_by_its_name() {
    let well = |path: &str, row, column| PlateWell {
        path: path.into(),
        row,
        column,
    };
    let layout = OmePlate {
        rows: vec!["A".into(), "B".into()],
        columns: vec!["1".into(), "2".into()],
        wells: vec![well("A/1", 0, 0), well("B/2", 1, 1)],
        ..Default::default()
    };
    let plate = PlateState::new(
        "https://x/plate.zarr",
        StoreKind::RemoteZarr,
        Arc::new(layout),
    );
    let mut app = OctantApp {
        plate: Some(plate),
        ..Default::default()
    };
    let outcomes = app.run_utterance("well B 2");
    assert_eq!(outcomes, vec![Ok(Done::WellPicked("B2".into()))]);
    assert_eq!(app.plate.as_ref().and_then(|p| p.well), Some(1));
}
