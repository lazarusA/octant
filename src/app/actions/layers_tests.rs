//! Overlays, layer visibility, colors, the export dialog and plate wells.

use std::sync::Arc;

use super::{AppAction, Done, Failure, NameKind};
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

#[test]
fn plot_plots_even_with_the_add_overlay_toggle_on() {
    let mut app = plotted_app();
    app.layout.plot_as_overlay = true;
    let outcomes = app.run_utterance("plot sst");
    assert!(
        matches!(outcomes.last(), Some(Ok(Done::Plotted { .. }))),
        "{outcomes:?}"
    );
    assert!(app.layers.overlays().is_empty(), "plotted, not overlaid");
}

#[test]
fn jpeg_is_said_either_way() {
    let mut app = plotted_app();
    app.run_utterance("export as jpeg");
    assert_eq!(app.export_settings.format, ExportFormat::Jpeg);
}

/// An app browsing a plate of one well, `A/1`, whose fields are `fields`.
fn one_well_plate(fields: crate::app::state::plate::WellFields) -> OctantApp {
    let layout = OmePlate {
        rows: vec!["A".into()],
        columns: vec!["1".into()],
        wells: vec![PlateWell {
            path: "A/1".into(),
            row: 0,
            column: 0,
        }],
        ..Default::default()
    };
    let mut plate = PlateState::new(
        "https://x/plate.zarr",
        StoreKind::RemoteZarr,
        Arc::new(layout),
    );
    plate.set_well_fields("A/1", fields);
    OctantApp {
        plate: Some(plate),
        ..Default::default()
    }
}

#[test]
fn a_well_whose_read_failed_is_read_again_once() {
    use crate::app::state::plate::WellFields;
    let mut app = one_well_plate(WellFields::Failed);
    assert_eq!(app.run_utterance("well A1 field 1"), vec![], "read again");
    let plate = app.plate.as_mut().expect("plate");
    assert_eq!(plate.well_fields("A/1"), &WellFields::Loading);
    // Failing again, its toast says so: the command stops.
    plate.set_well_fields("A/1", WellFields::Failed);
    assert_eq!(app.resume_commands(), vec![Err(Failure::Refused)]);
}

#[test]
fn a_field_waits_for_its_well_to_be_read() {
    use crate::app::state::plate::WellFields;
    use crate::data::ome::OmeWell;
    use crate::data::ome::plate::WellField;
    let mut app = one_well_plate(WellFields::Loading);
    // No dataset open asks for one; another dataset than a plate finds no
    // well; a plate's field without a well picked asks for a well.
    let mut no_plate = OctantApp::default();
    let well = AppAction::OpenPlateWell {
        well: Some("B3".into()),
        field: Some(1),
    };
    assert_eq!(no_plate.dispatch(well.clone()), Err(Failure::NoDataset));
    no_plate.selected.metadata = Some(Default::default());
    assert!(matches!(
        no_plate.dispatch(well),
        Err(Failure::NotFound {
            kind: NameKind::Well,
            ..
        })
    ));
    assert_eq!(
        app.run_utterance("field 1"),
        vec![Err(Failure::NoWellPicked)]
    );
    assert_eq!(
        app.run_utterance("well A1 field 1"),
        vec![],
        "waits for the well's fields"
    );
    let field = WellField {
        path: "0".into(),
        acquisition: None,
    };
    let ready = WellFields::Ready(Arc::new(OmeWell {
        fields: vec![field],
    }));
    if let Some(plate) = &mut app.plate {
        plate.set_well_fields("A/1", ready);
    }
    let outcomes = app.resume_commands();
    assert!(
        matches!(outcomes.as_slice(), [Ok(Done::Opening(_))]),
        "{outcomes:?}"
    );
}
