//! Playback keys on an animated plot: step, jump to the ends, play and loop,
//! and stay out of the way while a widget holds focus.

use super::dispatch;
use crate::app::OctantApp;
use crate::app::test_support::{make_resident, memory_app, selection_of};
use crate::data::SliceRequest;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2, vec2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

/// `sst(time, lat, lon)` plotted along its three steps, every one resident.
fn animated_app() -> OctantApp {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "sst");
    *app.layers.base.selection_mut() = selection.clone();
    app.selected = selection;
    make_resident(&mut app, &SliceRequest::full_range("sst", &[3, 5, 4]));
    app
}

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| dispatch(app, ui.ctx(), true)
}

#[test]
fn keys_step_and_jump_along_the_animated_dimension() {
    let mut app = animated_app();
    let mut h = Harness::new(SCREEN);
    let mut step_after = |app: &mut OctantApp, key| {
        h.key(key, Modifiers::NONE, &mut run(app));
        app.playback.current_timestep
    };
    assert_eq!(step_after(&mut app, Key::Period), 1);
    assert_eq!(step_after(&mut app, Key::End), 2);
    assert_eq!(step_after(&mut app, Key::Home), 0);
    assert_eq!(step_after(&mut app, Key::Comma), 2, "steps back around");
}

#[test]
fn space_plays_and_l_loops() {
    let mut app = animated_app();
    let looping = app.playback.loop_playback;
    let mut h = Harness::new(SCREEN);
    h.key(Key::Space, Modifiers::NONE, &mut run(&mut app));
    assert!(app.playback.is_playing);
    h.key(Key::L, Modifiers::NONE, &mut run(&mut app));
    assert_ne!(app.playback.loop_playback, looping);
    h.key(Key::Space, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.playback.is_playing);
}

#[test]
fn a_focused_button_keeps_space() {
    let mut app = animated_app();
    let mut h = Harness::new(SCREEN);
    let button = egui::Id::unique("a focused button");
    let mut run = |ui: &mut Ui| {
        dispatch(&mut app, ui.ctx(), true);
        let rect = Rect::from_min_size(pos2(5.0, 5.0), vec2(40.0, 20.0));
        ui.interact(rect, button, egui::Sense::click());
    };
    h.settle(1, &mut run);
    h.ctx.memory_mut(|m| m.request_focus(button));
    h.key(Key::Space, Modifiers::NONE, &mut run);
    h.key(Key::Period, Modifiers::NONE, &mut run);
    drop(run);
    assert!(!app.playback.is_playing);
    assert_eq!(app.playback.current_timestep, 0);
}
