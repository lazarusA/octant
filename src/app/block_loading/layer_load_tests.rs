//! Overlays request, receive and show their own blocks at their own step, leaving the
//! base layer alone, and follow the base layer's step.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, Source};
use crate::app::test_support::{
    TARGET, add_memory_dataset, indices, make_resident, memory_app, poll_until, selection_of,
};
use crate::data::SliceRequest;

/// Asserts that the base layer requested and shows nothing.
fn assert_base_untouched(app: &OctantApp) {
    let base = &app.layers.base;
    assert!(base.data.matrix.is_none(), "the base layer shows nothing");
    assert!(base.load.slice_request.is_none());
    assert!(base.load.block_key.is_none());
}

#[test]
fn a_resident_overlay_block_is_shown_in_the_overlay_only() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "elev");
    let id = app.layers.push(Source::Variable(selection));
    let request = app.staged_layer_request(id).expect("overlay request");
    make_resident(&mut app, &request.request);

    app.load_layer_block(id);

    let overlay = app.layers.get(id).expect("overlay");
    assert!(overlay.data.matrix.is_some(), "the overlay shows its block");
    assert_eq!(overlay.load.pending_target_step, None);
    let shown = overlay
        .load
        .slice_request
        .as_ref()
        .map(|r| r.variable.as_str());
    assert_eq!(shown, Some(request.var.name.as_str()));
    assert_base_untouched(&app);
}

#[test]
fn a_fetched_overlay_block_is_routed_to_the_overlay() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "elev");
    let id = app.layers.push(Source::Variable(selection));

    app.load_layer_block(id);
    let overlay = app.layers.get(id).expect("overlay");
    assert!(
        overlay.load.block_key.is_some(),
        "the overlay requests its block"
    );
    assert_eq!(overlay.load.pending_target_step, Some(0));
    assert_base_untouched(&app);

    poll_until(&mut app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
    let overlay = app.layers.get(id).expect("overlay");
    assert!(overlay.load.block_key.is_none(), "the request is settled");
    assert_base_untouched(&app);
}

#[test]
fn an_overlay_past_its_extent_shows_its_last_step() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "t2m");
    let id = app.layers.push(Source::Variable(selection));
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    // The base layer's longer animated dimension is at step 7.
    app.playback.current_timestep = 7;

    app.load_layer_block(id);

    assert_eq!(
        app.playback.current_timestep, 7,
        "an overlay never moves the shared step"
    );
    assert_eq!(app.layer_step(id), 2);
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_config[0].index, 2);
    assert!(
        overlay.data.matrix.is_some(),
        "the clamped step is projected"
    );
}

#[test]
fn a_base_step_that_arrives_reloads_the_animated_overlays() {
    let (mut app, meta) = memory_app();
    let base = selection_of(&mut app, &meta, "t2m");
    let overlay = selection_of(&mut app, &meta, "sst");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    let id = app.layers.push(Source::Variable(overlay));
    // Every step of the overlay is resident; the base layer's are fetched.
    make_resident(&mut app, &SliceRequest::full_range("sst", &[3, 5, 4]));

    app.request_step_or_load(1);
    assert_eq!(
        app.playback.current_timestep, 0,
        "the base step is still loading"
    );
    poll_until(&mut app, |app| app.playback.current_timestep == 1);

    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_config[0].index, 1);
    assert!(
        overlay.data.matrix.is_some(),
        "the overlay shows the new step"
    );
}

#[test]
fn a_staged_variable_keeps_its_own_selections_over_the_plotted_animated_one() {
    let (mut app, meta) = memory_app();
    // `t2m` is plotted, animated along time (dimension 0), at step 2.
    let plotted = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = plotted;
    app.playback.current_timestep = 2;
    // `rgb` (band, lat, lon) is staged: it has no animated dimension.
    app.selected = selection_of(&mut app, &meta, "rgb");
    let staged = indices(&app.selected);
    let expected = app.staged_layer_request(LayerId::BASE).expect("request");

    app.load_selected_variable_block();

    let request = app
        .layers
        .base
        .load
        .slice_request
        .as_ref()
        .expect("request");
    assert_eq!(request.selections, expected.request.selections);
    assert_eq!(indices(&app.selected), staged, "the staged indices stay");
}

#[test]
fn browsing_another_dataset_leaves_the_plot_until_it_is_plotted() {
    const OTHER: &str = "memory://other";
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.plotted().metadata.is_some());

    // Open another dataset and stage one of its variables, unplotted.
    let other = add_memory_dataset(&mut app, OTHER);
    app.selected = selection_of(&mut app, &other, "elev");
    app.selected.store_target = OTHER.to_string();
    assert!(app.is_exploring_unplotted_variable());
    assert_eq!(app.plotted().store_target, TARGET, "the plot stays");
    assert_eq!(
        app.effective_animated_dim(),
        Some(0),
        "and so does its time"
    );

    app.plot_selection();
    poll_until(&mut app, |app| app.plotted().store_target == OTHER);
    let shown = app
        .effective_variable_info()
        .map(|v| v.name.trim_matches('/'));
    assert_eq!(shown, Some("elev"), "plotting makes it the active one");
    assert!(!app.is_exploring_unplotted_variable());
}

#[test]
fn the_plot_never_takes_the_staged_variables_animated_dimension() {
    let (mut app, meta) = memory_app();
    *app.layers.base.selection_mut() = selection_of(&mut app, &meta, "elev");
    app.selected = selection_of(&mut app, &meta, "t2m");
    assert_eq!(app.effective_animated_dim(), None, "elev has no time");
    assert_eq!(app.layer_animated_dim(LayerId::BASE), None);
}

#[test]
fn a_store_that_cannot_open_says_so() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "elev");
    app.selected.store_kind = crate::app::StoreKind::LocalGeoTiff;
    app.selected.store_target = "/nonexistent/octant-b9.tif".into();

    app.load_selected_variable_block();

    let severity = app.toasts.items().first().map(|t| t.notice.severity);
    assert_eq!(severity, Some(crate::ui::toast::Severity::Error));
}

#[test]
fn browsing_a_variable_that_cannot_be_a_volume_leaves_the_plotted_volume() {
    use crate::app::SpatialRole;
    use crate::plots::PlotType;
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.switch_plot_type(PlotType::Volume);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.volume.is_some());
    // A staged volume over a discrete grid, which no volume can draw.
    let mut staged = selection_of(&mut app, &meta, "sst");
    staged.dim_config[2].spatial = SpatialRole::Grid;
    staged.plot_type = PlotType::Volume;
    app.selected = staged;

    // A block of the plotted t2m arrives (its lookahead) while sst is staged.
    let source_id =
        crate::app::StoreKind::make_source_id(crate::app::StoreKind::RemoteZarr, TARGET);
    let store = app
        .dataset_manager
        .get(&source_id)
        .expect("store")
        .store
        .clone();
    let block = store
        .fetch(&SliceRequest::full_range("t2m", &[3, 5, 4]))
        .expect("block");
    app.apply_block_projection(LayerId::BASE, &block);

    assert_eq!(app.plotted().plot_type, PlotType::Volume, "the plot stays");
    assert_eq!(app.selected.plot_type, PlotType::Volume, "and so does sst");
}
