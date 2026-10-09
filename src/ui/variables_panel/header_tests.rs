//! The Dimensions panel header: the Add Overlay toggle, then Plot Data, adds
//! the staged variable as an overlay once; the toggle stays off while it can't.

use super::show_variable_controls;
use crate::app::OctantApp;
use crate::app::test_support::{make_resident, memory_app, selection_of};
use crate::data::SliceRequest;
use crate::ui::test_input::Harness;
use egui::epaint::ClippedShape;
use egui::{Pos2, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

/// One frame of `app`'s Dimensions panel.
fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_variable_controls(app, ui.ctx(), SCREEN)
}

/// The center of the painted text `label` in `shapes`, if any.
fn text_center(shapes: &[ClippedShape], label: &str) -> Option<Pos2> {
    shapes.iter().find_map(|clipped| match &clipped.shape {
        egui::Shape::Text(text) if text.galley.text() == label => {
            Some(text.pos + text.galley.size() * 0.5)
        }
        _ => None,
    })
}

/// Clicks the painted text `label` of the panel; whether it was found.
fn click(app: &mut OctantApp, label: &str) -> bool {
    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(3, &mut run(app));
    let Some(pos) = text_center(&shapes, label) else {
        return false;
    };
    h.pointer(pos, Some(true), &mut run(app));
    h.pointer(pos, Some(false), &mut run(app));
    true
}

/// `t2m` plotted from resident blocks, `sst` resident and staged in the panel.
fn plotted_with_sst_staged() -> OctantApp {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    app.selected = selection_of(&mut app, &meta, "sst");
    app.show_variable_controls = true;
    app
}

#[test]
fn add_overlay_then_plot_data_adds_the_overlay() {
    let mut app = plotted_with_sst_staged();
    let base_var = app.plotted().variable_idx;

    assert!(click(&mut app, "Add Overlay"), "the toggle is shown");
    assert!(app.plot_as_overlay, "the toggle is on");
    assert!(app.layers.overlays().is_empty(), "nothing is added yet");

    assert!(click(&mut app, "Plot Data"));
    assert_eq!(app.layers.overlays().len(), 1, "added as an overlay");
    assert!(!app.plot_as_overlay, "the toggle turns off");
    assert_eq!(app.plotted().variable_idx, base_var, "the plot stays");
}

#[test]
fn add_overlay_toggles_off_again() {
    let mut app = plotted_with_sst_staged();
    click(&mut app, "Add Overlay");
    click(&mut app, "Add Overlay");
    assert!(!app.plot_as_overlay);
}

#[test]
fn a_variable_overlays_only_once() {
    let mut app = plotted_with_sst_staged();
    app.plot_as_overlay = true;
    app.plot_from_panel();
    assert_eq!(app.layers.overlays().len(), 1);

    assert!(
        app.overlay_unavailable_for(app.selected.variable_idx)
            .is_some()
    );
    click(&mut app, "Add Overlay");
    assert!(!app.plot_as_overlay, "the toggle is disabled for it");
    app.plot_as_overlay = true;
    assert!(app.add_overlay(app.selected.variable_idx).is_none());
    assert_eq!(app.layers.overlays().len(), 1);

    let base_var = app.plotted().variable_idx;
    assert!(
        app.overlay_unavailable_for(base_var).is_some(),
        "nor the plot's own"
    );
}

#[test]
fn add_overlay_stays_off_while_nothing_is_plotted() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.show_variable_controls = true;
    assert!(
        click(&mut app, "Add Overlay"),
        "the toggle is shown, disabled"
    );
    assert!(!app.plot_as_overlay);
    assert!(app.layers.overlays().is_empty());
}
