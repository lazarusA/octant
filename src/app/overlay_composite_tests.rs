//! Each overlay's composite: its defaults, toggling it without touching the
//! base layer, and its hover row.

use crate::app::overlays::MAX_OVERLAYS;
use crate::app::test_support::{make_resident, memory_app, poll_until, selection_of};
use crate::data::{DatasetMetadata, DimensionSelection, SliceRequest};
use crate::ui::hover::card::{HoverValue, LayerValue};
use crate::ui::hover::composite::CompositeKind;
use crate::ui::hover::overlays::{HoverAt, overlay_values};

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

/// Turns overlay `id`'s composite on with `bands` ticked and waits for it.
fn compose(app: &mut crate::app::OctantApp, id: crate::app::layers::LayerId, bands: &[usize]) {
    if let Some(layer) = app.layers.get_mut(id) {
        layer.composite.enabled = true;
        for cfg in &mut layer.composite.channel_configs {
            cfg.visible = bands.contains(&cfg.index);
        }
        layer.clear_2d();
    }
    app.load_layer_block(id);
    poll_until(app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
}

/// The band selection overlay `id` last requested.
fn requested_bands(
    app: &crate::app::OctantApp,
    id: crate::app::layers::LayerId,
) -> Option<DimensionSelection> {
    let layer = app.layers.get(id)?;
    layer
        .load
        .slice_request
        .as_ref()?
        .selections
        .first()
        .cloned()
}

#[test]
fn a_banded_overlay_starts_on_one_band() {
    let (app, id) = app_with_rgb_overlay();
    let overlay = app.layers.get(id).expect("overlay");
    assert!(!overlay.composite.enabled, "composites start off");
    assert_eq!(overlay.composite.channel_configs.len(), 3);
    assert_eq!(overlay.selection().dim_ranges[0], (0, 0));
    assert!(!app.layers.base.composite.enabled, "the base stays scalar");
    assert!(app.layer_has_rgb_bands(id));
    assert_eq!(app.layer_num_bands(id), 3);
}

#[test]
fn an_overlay_composite_reads_its_ticked_bands_and_keeps_them() {
    let (mut app, id) = app_with_rgb_overlay();
    let base_key = app.layers.base.load.slice_request.clone();
    compose(&mut app, id, &[0, 2]);
    let bands = Some(DimensionSelection::Range { start: 0, end: 3 });
    assert_eq!(requested_bands(&app, id), bands);
    assert_eq!(app.layers.base.load.slice_request, base_key);

    // The base window moving leaves the overlay's bands alone.
    app.sync_overlays_to_base();
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_ranges[0], (0, 2));

    // Off again, it reads its one band.
    if let Some(layer) = app.layers.get_mut(id) {
        layer.composite.enabled = false;
    }
    app.load_layer_block(id);
    assert_eq!(
        requested_bands(&app, id),
        Some(DimensionSelection::Index(0))
    );
}

#[test]
fn a_composite_overlay_row_names_its_composite() {
    let (mut app, id) = app_with_rgb_overlay();
    compose(&mut app, id, &[0, 1, 2]);
    let mut rows = [LayerValue::EMPTY; MAX_OVERLAYS];
    assert_eq!(overlay_values(&app, at((1, 1)), &mut rows), 1);
    assert_eq!(rows[0].name, "rgb");
    assert_eq!(rows[0].value, HoverValue::Composite(CompositeKind::Overlay));
    assert!(app.layers.get(id).is_some_and(|l| l.composite.enabled));
}

/// The hover at base cell `cell`, off any detail tile.
fn at(cell: (usize, usize)) -> HoverAt {
    HoverAt {
        cell,
        norm: (-1.0, -1.0),
    }
}

#[test]
fn an_overlay_without_a_band_dimension_keeps_the_base_time_window() {
    let (mut app, meta) = memory_app();
    let mut base = selection_of(&mut app, &meta, "t2m");
    let time = base.animated_dim.expect("time");
    base.dim_ranges[time] = (0, 2);
    base.dim_config[time].range = (0, 2);
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    app.load_layer_block(id);
    let overlay = app.layers.get(id).expect("overlay");
    // `time` is dimension 0, where `layer_channel_dim` falls back to.
    assert_eq!(overlay.selection().dim_ranges[0], (0, 2));
}

#[test]
fn a_band_shared_with_the_base_follows_its_slider() {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("rgb", &[3, 5, 4]));
    let mut base = selection_of(&mut app, &meta, "rgb");
    base.dim_ranges[0] = (1, 2);
    base.dim_config[0].range = (1, 2);
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();

    // The same arrays in another store, overlaid on the plotted `rgb`.
    let other = crate::app::test_support::add_memory_dataset(&mut app, "memory://other");
    let mut staged = selection_of(&mut app, &other, "rgb");
    staged.store_target = "memory://other".into();
    app.selected = staged;
    let id = app.add_overlay(index_of(&other, "rgb")).expect("overlay");
    assert_eq!(app.overlay_own_channel(id), None, "the base has `band`");

    // Ticked channels outside the base's bands don't widen them.
    if let Some(layer) = app.layers.get_mut(id) {
        layer.composite.enabled = true;
        layer
            .composite
            .channel_configs
            .iter_mut()
            .for_each(|c| c.visible = true);
    }
    app.fit_overlay_channels(id);
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_ranges[0], (1, 2));
}
