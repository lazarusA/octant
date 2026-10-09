//! The colorbar controls driven headlessly: shown only while hovered and
//! never during export; the grip drags and resets the panel, the flip turns
//! it.

use super::controls;
use super::show_colorbar_overlay;
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use egui::{Event, PointerButton, Pos2, RawInput, Rect, Vec2, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

struct Harness {
    ctx: egui::Context,
    time: f64,
}

impl Harness {
    /// A harness that has laid the colorbar out over a few frames.
    fn new(app: &mut OctantApp) -> Self {
        let mut h = Self {
            ctx: egui::Context::default(),
            time: 0.0,
        };
        for _ in 0..3 {
            h.frame(app, Vec::new());
        }
        h
    }

    /// Runs one frame with `events`; how many circles were painted (the
    /// grip's dots).
    fn frame(&mut self, app: &mut OctantApp, events: Vec<Event>) -> usize {
        self.time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(SCREEN),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self
            .ctx
            .run_ui(input, |ui| show_colorbar_overlay(app, ui.ctx(), SCREEN));
        output.textures_delta.clear();
        output
            .shapes
            .iter()
            .filter(|c| matches!(c.shape, egui::Shape::Circle(_)))
            .count()
    }

    /// The base layer's colorbar panel.
    fn panel(&self) -> Rect {
        let id = egui::Id::new(("octant_colorbar_overlay", LayerId::BASE));
        self.ctx
            .memory(|m| m.area_rect(id))
            .expect("the base colorbar is shown")
    }

    fn press(&mut self, app: &mut OctantApp, pos: Pos2, pressed: bool) {
        let event = Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        self.frame(app, vec![Event::PointerMoved(pos), event]);
    }

    fn click(&mut self, app: &mut OctantApp, pos: Pos2) {
        self.frame(app, vec![Event::PointerMoved(pos)]);
        self.press(app, pos, true);
        self.press(app, pos, false);
    }
}

fn app() -> OctantApp {
    OctantApp {
        show_colorbar: true,
        ..Default::default()
    }
}

#[test]
fn controls_show_only_while_the_panel_is_hovered() {
    let mut app = app();
    let mut h = Harness::new(&mut app);
    assert_eq!(
        h.frame(&mut app, vec![Event::PointerMoved(pos2(5.0, 5.0))]),
        0
    );
    let inside = h.panel().center();
    assert!(h.frame(&mut app, vec![Event::PointerMoved(inside)]) >= 6);
}

#[test]
fn controls_hide_during_export() {
    let mut app = app();
    let mut h = Harness::new(&mut app);
    app.pending_export = Some(crate::export::PendingExportRequest {
        format: Default::default(),
        target: Default::default(),
        roi: Default::default(),
        jpeg_quality: 90,
        copy_to_clipboard: false,
        output_path: None,
        canvas_rect_in_points: Rect::NOTHING,
        pixels_per_point: 1.0,
    });
    let inside = h.panel().center();
    assert_eq!(h.frame(&mut app, vec![Event::PointerMoved(inside)]), 0);
}

#[test]
fn dragging_the_grip_moves_the_panel_and_double_click_resets_it() {
    let mut app = app();
    let mut h = Harness::new(&mut app);
    let before = h.panel();
    let [grip, _] = controls::rects(before);
    let start = grip.center();
    h.frame(&mut app, vec![Event::PointerMoved(start)]);
    h.press(&mut app, start, true);
    let delta = Vec2::new(-200.0, -300.0);
    for step in 1..=10 {
        let at = start + delta * (step as f32 / 10.0);
        h.frame(&mut app, vec![Event::PointerMoved(at)]);
    }
    h.press(&mut app, start + delta, false);
    h.frame(&mut app, Vec::new());

    assert!(app.layers.base.colorbar.pos.is_some());
    let moved = h.panel().center_bottom() - before.center_bottom();
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let [grip, _] = controls::rects(h.panel());
    h.click(&mut app, grip.center());
    h.click(&mut app, grip.center());
    assert_eq!(app.layers.base.colorbar.pos, None);
}

#[test]
fn the_flip_turns_the_bar() {
    let mut app = app();
    let mut h = Harness::new(&mut app);
    let [_, flip] = controls::rects(h.panel());
    h.click(&mut app, flip.center());
    assert_eq!(
        app.layers.base.colorbar.orientation,
        BarOrientation::Vertical
    );
    for _ in 0..3 {
        h.frame(&mut app, Vec::new());
    }
    let panel = h.panel();
    assert!(panel.height() > panel.width(), "vertical panel: {panel:?}");
    assert!(SCREEN.contains_rect(panel));
}
