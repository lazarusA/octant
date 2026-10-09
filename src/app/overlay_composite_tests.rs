//! Each overlay's composite: its defaults, toggling it without touching the
//! base layer, and its hover row.

use crate::app::overlays::MAX_OVERLAYS;
use crate::app::test_support::{make_resident, memory_app, poll_until, selection_of};
use crate::data::{DatasetMetadata, SliceRequest};
use crate::ui::hover::card::{HoverValue, LayerValue};
use crate::ui::hover::composite::CompositeKind;
use crate::ui::hover::overlays::overlay_values;

fn index_of(meta: &DatasetMetadata, name: &str) -> usize {
    meta.variables
        .iter()
        .position(|v| v.name.trim_matches('/') == name)
        .expect("variable")
}

/// `t2m` plotted as a heatmap, with the three-band `rgb` added as an overlay.
fn app_with_rgb_overlay() -> (crate::app::OctantApp, crate::app::layers::LayerId) {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    make_resident(&mut app, &SliceRequest::full_range("rgb", &[3, 5, 4]));
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    let id = app.add_overlay(index_of(&meta, "rgb")).expect("overlay");
    (app, id)
}

#[test]
fn a_banded_overlay_starts_as_its_own_composite() {
    let (app, id) = app_with_rgb_overlay();
    let overlay = app.layers.get(id).expect("overlay");
    assert!(overlay.composite.enabled, "its band dimension composes");
    assert_eq!(overlay.composite.channel_configs.len(), 3);
    assert!(!app.layers.base.composite.enabled, "the base stays scalar");
    assert!(app.layer_has_rgb_bands(id));
    assert_eq!(app.layer_num_bands(id), 3);
}

#[test]
fn turning_an_overlay_composite_off_reloads_only_that_overlay() {
    let (mut app, id) = app_with_rgb_overlay();
    let base_key = app.layers.base.load.slice_request.clone();
    if let Some(layer) = app.layers.get_mut(id) {
        layer.composite.enabled = false;
    }
    app.load_layer_block(id);
    poll_until(&mut app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });

    let overlay = app.layers.get(id).expect("overlay");
    let band = overlay
        .load
        .slice_request
        .as_ref()
        .and_then(|r| r.selections.first().cloned());
    assert_eq!(
        band,
        Some(crate::data::DimensionSelection::Index(0)),
        "one band is read once the composite is off"
    );
    assert_eq!(app.layers.base.load.slice_request, base_key);
}

#[test]
fn a_composite_overlay_row_names_its_composite() {
    let (app, id) = app_with_rgb_overlay();
    let mut rows = [LayerValue::EMPTY; MAX_OVERLAYS];
    assert_eq!(overlay_values(&app, (1, 1), &mut rows), 1);
    assert_eq!(rows[0].name, "rgb");
    assert_eq!(rows[0].value, HoverValue::Composite(CompositeKind::Overlay));
    assert!(app.layers.get(id).is_some_and(|l| l.composite.enabled));
}
