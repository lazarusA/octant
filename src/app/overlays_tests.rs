//! Adding overlays over a plotted heatmap, following the base window,
//! removing them, per-layer colormaps and their hover readings.

use crate::app::OctantApp;
use crate::app::layers::{Alignment, LayerId};
use crate::app::overlays::MAX_OVERLAYS;
use crate::app::test_support::{make_resident, memory_app, poll_until, selection_of};
use crate::data::{DatasetMetadata, SliceRequest};
use crate::ui::hover::card::LayerValue;
use crate::ui::hover::overlays::overlay_values;
use crate::utils::colormap::registry;

fn index_of(meta: &DatasetMetadata, name: &str) -> usize {
    meta.variables
        .iter()
        .position(|v| v.name.trim_matches('/') == name)
        .expect("variable")
}

/// `t2m` plotted as a heatmap from resident blocks, `sst` resident too.
fn plotted_app() -> (OctantApp, DatasetMetadata) {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    assert!(app.layers.base.data.matrix.is_some(), "t2m is plotted");
    (app, meta)
}

#[test]
fn an_added_overlay_reads_the_base_window_and_draws() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.alignment, Alignment::SameGrid);
    assert!(overlay.is_drawn());
    assert!(overlay.data.matrix.is_some(), "its resident block is shown");
    assert_eq!(overlay.selection().dim_ranges, app.plotted().dim_ranges);
    assert!(overlay.color.opacity < 1.0, "the base shows through");
}

#[test]
fn no_overlay_is_added_without_a_plotted_heatmap() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    assert!(app.overlay_unavailable().is_some());
    assert_eq!(app.add_overlay(index_of(&meta, "sst")), None);
    assert!(app.layers.overlays().is_empty());
}

#[test]
fn at_most_max_overlays_are_added() {
    let (mut app, meta) = plotted_app();
    assert!(app.add_overlay(index_of(&meta, "sst")).is_some());
    for _ in 1..MAX_OVERLAYS {
        app.layers.push(crate::app::layers::Source::default());
    }
    assert!(app.overlay_unavailable().is_some(), "the stack is full");
    assert_eq!(app.add_overlay(index_of(&meta, "elev")), None);
    assert_eq!(app.layers.overlays().len(), MAX_OVERLAYS);
}

#[test]
fn overlays_follow_a_new_base_window() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let lon = 2;
    app.selected.dim_ranges[lon] = (0, 1);
    app.selected.dim_config[lon].range = (0, 1);
    app.plot_selection();
    let width = |app: &OctantApp, id: LayerId| {
        let layer = app.layers.get(id)?;
        layer.data.matrix.as_ref().map(|m| m.width)
    };
    // Both narrower windows are fetched; the overlay's once the base's arrived.
    poll_until(&mut app, |app| width(app, id) == Some(2));

    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_ranges[lon], (0, 1));
    assert_eq!(overlay.alignment, Alignment::SameGrid);
    assert_eq!(width(&app, LayerId::BASE), Some(2));
}

#[test]
fn the_picker_edits_its_target_overlay_until_it_is_removed() {
    let _registry = registry::test_lock();
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let set1 = registry::find("colorbrewer:Set1").unwrap_or(0);

    app.colormaps.target = Some(id);
    assert_eq!(app.picker_layer(), id);
    app.picker_style_mut().colormap = set1;
    app.picker_style_mut().reversed = true;
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(
        (overlay.color.colormap, overlay.color.reversed),
        (set1, true)
    );
    assert_ne!(app.layers.base.color.colormap, set1);
    assert!(!app.layers.base.color.reversed);

    app.preview_colormap = Some(registry::default_id());
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(
        app.layer_colormap(overlay),
        registry::default_id(),
        "preview"
    );
    assert_eq!(app.effective_colormap(), app.layers.base.color.colormap);

    app.remove_overlay(id);
    assert!(app.layers.overlays().is_empty());
    assert_eq!(app.colormaps.target, None);
    assert_eq!(app.picker_layer(), LayerId::BASE);
}

#[test]
fn the_hover_reads_each_drawn_overlay_at_the_base_cell() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let hidden = app.add_overlay(index_of(&meta, "elev")).expect("overlay");
    if let Some(layer) = app.layers.get_mut(hidden) {
        layer.visible = false;
    }
    let mut rows = [LayerValue::EMPTY; MAX_OVERLAYS];
    let count = overlay_values(&app, (2, 3), &mut rows);
    assert_eq!(count, 1, "hidden overlays have no row");
    let matrix = app.layers.get(id).and_then(|l| l.data.matrix.as_ref());
    let expected = matrix.and_then(|m| m.values.get(3 * m.width + 2)).copied();
    assert_eq!(rows[0].name, "sst");
    assert_eq!(
        rows[0].value,
        crate::ui::hover::card::HoverValue::from_raw(expected.unwrap_or(f32::NAN), None)
    );
}

#[test]
fn an_overlay_opacity_curve_is_its_own_until_removed() {
    let _registry = registry::test_lock();
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let key = id.key();
    if let Some(layer) = app.layers.get_mut(id) {
        layer.color.alpha.text = "0, 1".to_string();
    }
    app.apply_alpha_curve(id);

    let overlay = app.layers.get(id).expect("overlay");
    let row = overlay.color.alpha_row();
    assert!(row.is_some(), "the overlay has a curve row");
    assert_eq!(app.get_color_params(overlay).alpha_row, row.unwrap_or(0));
    assert_eq!(app.layers.base.color.alpha_row(), None, "the base has none");
    assert!(!app.layers.base.color.is_translucent());

    app.remove_overlay(id);
    assert_eq!(registry::alpha_row(key), None, "its row is freed");
}

#[test]
fn an_overlay_color_range_resets_to_its_own_data() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let base_range = (
        app.layers.base.color.range_min,
        app.layers.base.color.range_max,
    );
    let Some(layer) = app.layers.get_mut(id) else {
        panic!("overlay");
    };
    let extent = layer.data.matrix.as_ref().map(|m| (m.min_val, m.max_val));
    (layer.color.range_min, layer.color.range_max) = (-5.0, 5.0);
    layer.color.lock_bounds = true;

    app.reset_layer_color_range(id);
    let color = &app.layers.get(id).expect("overlay").color;
    assert_eq!(Some((color.range_min, color.range_max)), extent);
    assert!(!color.lock_bounds);
    let base = &app.layers.base.color;
    assert_eq!(
        (base.range_min, base.range_max),
        base_range,
        "the base keeps its range"
    );
}

#[test]
fn every_overlay_colormap_is_a_sequential_map() {
    use crate::app::overlays::OVERLAY_COLORMAPS;
    for key in OVERLAY_COLORMAPS {
        let id = registry::find(key).unwrap_or_else(|| panic!("{key} is in the catalog"));
        let kind = registry::with_entry(id, |e| e.kind);
        assert_eq!(
            kind,
            Some(crate::utils::colormap::ColormapKind::Sequential),
            "{key}"
        );
    }
}

#[test]
fn new_overlays_take_unused_colormaps_of_the_set() {
    use crate::app::overlays::OVERLAY_COLORMAPS;
    let _registry = registry::test_lock();
    let (mut app, meta) = plotted_app();
    let base = app.layers.base.color.colormap;
    let first = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let second = app.add_overlay(index_of(&meta, "elev")).expect("overlay");
    let colormap = |app: &OctantApp, id| app.layers.get(id).map(|l| l.color.colormap);
    assert_eq!(colormap(&app, first), registry::find(OVERLAY_COLORMAPS[0]));
    assert_eq!(colormap(&app, second), registry::find(OVERLAY_COLORMAPS[1]));
    assert_eq!(
        app.layers.base.color.colormap, base,
        "the base keeps its colormap"
    );

    app.remove_overlay(first);
    let again = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    assert_eq!(
        colormap(&app, again),
        registry::find(OVERLAY_COLORMAPS[0]),
        "freed maps are reused"
    );
}
