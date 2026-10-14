//! Pyramids are decided per layer: Resampling ticked, or the layer's own data
//! too large for one GPU buffer; nothing turns the setting on by itself.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::pipeline::needs_pyramid;
use crate::app::test_support::{make_resident, memory_app, selection_of};
use crate::data::SliceRequest;
use crate::plots::limits::max_storage_elements;

/// `t2m` plotted from resident blocks with `sst` added as an overlay.
fn app_with_overlay() -> (OctantApp, LayerId) {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    let sst = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "sst")
        .expect("sst");
    let id = app.add_overlay(sst).expect("overlay");
    (app, id)
}

fn has_pyramid(app: &OctantApp, id: LayerId) -> bool {
    app.layers
        .get(id)
        .is_some_and(|layer| layer.data.pyramid.is_some())
}

#[test]
fn only_ticked_or_oversized_data_is_resampled() {
    assert!(!needs_pyramid(false, max_storage_elements()));
    assert!(needs_pyramid(false, max_storage_elements() + 1));
    assert!(needs_pyramid(true, 4));
}

#[test]
fn small_layers_have_no_pyramid_until_resampling_is_ticked() {
    let (mut app, overlay) = app_with_overlay();
    assert!(!has_pyramid(&app, LayerId::BASE));
    assert!(!has_pyramid(&app, overlay));

    app.enable_pyramid_resampling = true;
    app.rebuild_layer_pyramids();
    assert!(has_pyramid(&app, LayerId::BASE));
    assert!(has_pyramid(&app, overlay));
    let base = &app.layers.base.data;
    assert!(base.matrix.is_some(), "the layer keeps its matrix");
    let level0 = base.pyramid.as_ref().and_then(|p| p.levels.first());
    assert!(level0.is_some_and(|l| l.values.is_empty()), "no copy of it");

    app.enable_pyramid_resampling = false;
    app.rebuild_layer_pyramids();
    assert!(!has_pyramid(&app, LayerId::BASE));
    assert!(!has_pyramid(&app, overlay));
    assert!(!app.enable_pyramid_resampling, "rebuilds never tick it");
}
