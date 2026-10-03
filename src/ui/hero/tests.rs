//! Intake field focus on the hero landing, driven headlessly.

use super::chip_nav::chip_id;
use super::focus::intake_id;
use super::show_hero_landing;
use crate::app::OctantApp;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, pos2};

/// Run one frame; `hero` false skips drawing it, like a plotted canvas does.
fn frame(ctx: &egui::Context, app: &mut OctantApp, events: Vec<Event>, hero: bool) {
    let input = RawInput {
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| {
        if hero {
            show_hero_landing(app, ui);
        }
    });
    out.textures_delta.clear();
}

fn click(ctx: &egui::Context, app: &mut OctantApp, pos: Pos2) {
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(ctx, app, vec![Event::PointerMoved(pos)], true);
    frame(ctx, app, vec![button(true)], true);
    frame(ctx, app, vec![button(false)], true);
}

fn press(ctx: &egui::Context, app: &mut OctantApp, key: Key) {
    let event = Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    frame(ctx, app, vec![event], true);
}

fn focused(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.memory(|m| m.focused())
}

fn intake_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.has_focus(intake_id()))
}

fn drop_focus(ctx: &egui::Context) {
    ctx.memory_mut(|m| m.surrender_focus(intake_id()));
}

#[test]
fn intake_is_focused_on_first_load() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    frame(&ctx, &mut app, Vec::new(), true);
    frame(&ctx, &mut app, Vec::new(), true);
    assert!(intake_focused(&ctx));
}

#[test]
fn clicking_the_hero_background_refocuses_intake() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    frame(&ctx, &mut app, Vec::new(), true);
    drop_focus(&ctx);
    frame(&ctx, &mut app, Vec::new(), true);
    assert!(
        !intake_focused(&ctx),
        "no focus stealing while the hero stays up"
    );

    // Top-left corner: empty space above the centered cube.
    click(&ctx, &mut app, pos2(4.0, 4.0));
    assert!(intake_focused(&ctx));
}

#[test]
fn showing_the_hero_again_refocuses_intake() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    frame(&ctx, &mut app, Vec::new(), true);
    drop_focus(&ctx);

    frame(&ctx, &mut app, Vec::new(), false);
    frame(&ctx, &mut app, Vec::new(), true);
    assert!(intake_focused(&ctx));
}

#[test]
fn arrows_move_from_intake_through_the_chips_and_back() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    frame(&ctx, &mut app, Vec::new(), true);
    assert!(intake_focused(&ctx));

    press(&ctx, &mut app, Key::ArrowDown);
    assert_eq!(focused(&ctx), Some(chip_id(0)), "Down enters the chips");
    press(&ctx, &mut app, Key::ArrowRight);
    assert_eq!(focused(&ctx), Some(chip_id(1)));
    press(&ctx, &mut app, Key::End);
    assert_eq!(focused(&ctx), Some(chip_id(2)));
    press(&ctx, &mut app, Key::ArrowRight);
    assert_eq!(focused(&ctx), Some(chip_id(2)), "last chip stays put");
    press(&ctx, &mut app, Key::ArrowLeft);
    assert_eq!(focused(&ctx), Some(chip_id(1)));
    press(&ctx, &mut app, Key::Home);
    assert_eq!(focused(&ctx), Some(chip_id(0)));
    press(&ctx, &mut app, Key::ArrowUp);
    assert!(intake_focused(&ctx), "Up returns to the field");
}

#[test]
fn escape_on_a_chip_returns_to_intake() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    frame(&ctx, &mut app, Vec::new(), true);
    press(&ctx, &mut app, Key::ArrowDown);
    frame(&ctx, &mut app, Vec::new(), true);
    press(&ctx, &mut app, Key::Escape);
    assert!(intake_focused(&ctx));
}

/// One frame of a left panel (like the Dataset panel) drawn just before the
/// hero; returns the panel button's rect and whether it was clicked.
fn panel_frame(ctx: &egui::Context, app: &mut OctantApp, events: Vec<Event>) -> (egui::Rect, bool) {
    let input = RawInput {
        events,
        ..Default::default()
    };
    let mut button = (egui::Rect::NOTHING, false);
    let mut out = ctx.run_ui(input, |ui| {
        let mut show = true;
        egui::Panel::left("test_left_panel").show_collapsible(ui, &mut show, |ui| {
            let resp = ui.button("Load");
            button = (resp.rect, resp.clicked());
        });
        show_hero_landing(app, ui);
    });
    out.textures_delta.clear();
    button
}

#[test]
fn hero_background_does_not_cover_side_panels() {
    let ctx = egui::Context::default();
    let mut app = OctantApp::default();
    let (rect, _) = panel_frame(&ctx, &mut app, Vec::new());
    drop_focus(&ctx);

    let pos = rect.center();
    let press = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    panel_frame(&ctx, &mut app, vec![Event::PointerMoved(pos)]);
    panel_frame(&ctx, &mut app, vec![press(true)]);
    let (_, clicked) = panel_frame(&ctx, &mut app, vec![press(false)]);
    assert!(clicked, "the panel button receives its click");
    assert!(!intake_focused(&ctx), "the hero background did not take it");
}
