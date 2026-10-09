//! Overlays request, receive and show their own blocks at their own step, leaving the
//! base layer alone, and follow the base layer's step.

use crate::app::OctantApp;
use crate::app::layers::Source;
use crate::app::test_support::{make_resident, memory_app, poll_until, selection_of};
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
    app.current_timestep = 7;

    app.load_layer_block(id);

    assert_eq!(
        app.current_timestep, 7,
        "an overlay never moves the shared step"
    );
    assert_eq!(app.layer_step(id), 2);
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_indices.first(), Some(&2));
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
    assert_eq!(app.current_timestep, 0, "the base step is still loading");
    poll_until(&mut app, |app| app.current_timestep == 1);

    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_indices.first(), Some(&1));
    assert!(
        overlay.data.matrix.is_some(),
        "the overlay shows the new step"
    );
}
