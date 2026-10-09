//! The Dimensions panel header: its overlay toggle turns itself off while no
//! overlay can be added.

use super::show_variable_controls;
use crate::app::OctantApp;
use crate::app::test_support::{memory_app, selection_of};
use egui::{RawInput, Rect, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

fn run_frame(ctx: &egui::Context, app: &mut OctantApp) {
    let input = RawInput {
        screen_rect: Some(SCREEN),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| show_variable_controls(app, ui.ctx(), SCREEN));
    output.textures_delta.clear();
}

#[test]
fn the_overlay_toggle_is_off_while_nothing_is_plotted() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.show_variable_controls = true;
    app.plot_as_overlay = true;
    let ctx = egui::Context::default();
    run_frame(&ctx, &mut app);
    assert!(app.overlay_unavailable().is_some());
    assert!(!app.plot_as_overlay, "nothing to overlay yet");
}
