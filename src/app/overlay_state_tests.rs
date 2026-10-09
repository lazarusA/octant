//! Overlay state across the base plot's changes: pending coordinates, plot
//! type switches, hidden overlays, picker cleanup and window following.

use std::collections::HashMap;

use crate::app::layers::{Alignment, LayerId, follow_base_window};
use crate::app::test_support::{
    add_memory_dataset, make_resident, memory_app, poll_until, selection_of,
};
use crate::app::{OctantApp, StoreKind};
use crate::data::{CoordValues, DatasetMetadata, SliceRequest};
use crate::plots::PlotType;

const OTHER: &str = "memory://other";

fn index_of(meta: &DatasetMetadata, name: &str) -> usize {
    meta.variables
        .iter()
        .position(|v| v.name.trim_matches('/') == name)
        .expect("variable")
}

/// `t2m` plotted as a heatmap from resident blocks; `sst` and `elev`
/// resident too.
fn plotted_app() -> (OctantApp, DatasetMetadata) {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    make_resident(&mut app, &SliceRequest::full_range("elev", &[5, 4]));
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    (app, meta)
}

fn lat_lon() -> HashMap<String, CoordValues> {
    let regular = |start, step, len| CoordValues::Regular { start, step, len };
    HashMap::from([
        ("lat".to_string(), regular(-40.0, 20.0, 5)),
        ("lon".to_string(), regular(0.0, 90.0, 4)),
    ])
}

#[test]
fn an_overlay_waits_for_its_coordinates_then_draws() {
    let (mut app, _) = plotted_app();
    let other = add_memory_dataset(&mut app, OTHER);
    let mut staged = selection_of(&mut app, &other, "sst");
    staged.store_target = OTHER.to_string();
    app.selected = staged;

    let id = app
        .add_overlay(index_of(&other, "sst"))
        .expect("added, pending");
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.alignment, Alignment::IndexOnly);
    assert!(
        !overlay.is_drawn() && overlay.load.slice_request.is_none(),
        "nothing loads yet"
    );

    // Both datasets' coordinates arrive and match.
    for target in [crate::app::test_support::TARGET, OTHER] {
        let source = StoreKind::make_source_id(StoreKind::RemoteZarr, target);
        app.merge_variable_coordinates(&source, lat_lon());
    }
    assert_eq!(
        app.layers.get(id).map(|l| l.alignment),
        Some(Alignment::SameGrid)
    );
    poll_until(&mut app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
}

#[test]
fn switching_the_plot_type_reclassifies_overlays_at_once() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    // The volume's block is fetched, so no base sync reclassifies them.
    app.block_cache.clear();
    app.switch_plot_type(PlotType::Volume);
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.alignment, Alignment::Incompatible);
    assert!(!overlay.is_drawn());
}

#[test]
fn hidden_overlays_skip_steps_and_load_when_shown() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    app.set_layer_visible(id, false);
    let before = app
        .layers
        .get(id)
        .and_then(|l| l.load.slice_request.clone());

    app.current_timestep = 2;
    app.load_step_blocks();
    let after = app
        .layers
        .get(id)
        .and_then(|l| l.load.slice_request.clone());
    assert_eq!(after, before, "a hidden overlay loads nothing");
    assert!(!app.layers.drawn_ids().contains(&id));

    app.set_layer_visible(id, true);
    let shown = app
        .layers
        .get(id)
        .and_then(|l| l.load.slice_request.clone());
    assert_ne!(shown, before, "shown again, it loads the current step");
}

#[test]
fn removing_the_picked_overlay_drops_its_preview() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    app.colormaps.target = Some(id);
    app.preview_colormap = Some(0);
    app.remove_overlay(id);
    assert_eq!(app.preview_colormap, None);
    assert_eq!(app.picker_layer(), LayerId::BASE);
}

#[test]
fn following_the_base_window_ignores_playback_steps() {
    let (mut app, meta) = plotted_app();
    let id = app.add_overlay(index_of(&meta, "sst")).expect("overlay");
    let anim = app.plotted().animated_dim.expect("animated");
    let Some((base, overlay)) = app.layers.base_and_overlay_mut(id) else {
        panic!("overlay");
    };
    let mut base = base.selection().clone();
    base.dim_indices[anim] = 2;
    assert!(
        !follow_base_window(&base, overlay.selection_mut()),
        "a step is no move"
    );
    assert_eq!(
        overlay.selection().dim_indices[anim],
        2,
        "but it is followed"
    );

    let lon = 2;
    base.dim_ranges[lon] = (0, 1);
    assert!(
        follow_base_window(&base, overlay.selection_mut()),
        "a new window moves"
    );
    assert_eq!(overlay.selection().dim_ranges[lon], (0, 1));
}
