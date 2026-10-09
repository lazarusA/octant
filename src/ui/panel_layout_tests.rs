//! Docked panels line up and close up when one is dragged away; the Settings
//! panel's grip drags it and a double-click docks it again.

use crate::app::OctantApp;
use crate::ui::panel_layout::{self, GAP, Panel};
use crate::ui::settings::show_settings_window;
use egui::{Event, Id, PointerButton, Pos2, RawInput, Rect, Vec2, pos2};

const CANVAS: Rect = Rect::from_min_max(pos2(40.0, 30.0), pos2(1240.0, 930.0));

fn app() -> OctantApp {
    let mut app = OctantApp {
        show_variables_overlay: true,
        show_settings_panel: true,
        ..Default::default()
    };
    app.variables_overlay_width = 300.0;
    app.settings_overlay_width = 280.0;
    app
}

#[test]
fn docked_panels_line_up_from_the_canvas_corner() {
    let app = app();
    let corner = CANVAS.left_top() + Vec2::splat(GAP);
    assert_eq!(panel_layout::origin(&app, Panel::Variables, CANVAS), corner);
    let settings = panel_layout::origin(&app, Panel::Settings, CANVAS);
    assert_eq!(settings, corner + Vec2::new(300.0 + GAP, 0.0));
    let dims = panel_layout::origin(&app, Panel::Dimensions, CANVAS);
    assert_eq!(dims, corner + Vec2::new(300.0 + 280.0 + 2.0 * GAP, 0.0));
}

#[test]
fn hidden_or_dragged_panels_leave_the_row() {
    let mut app = app();
    let corner = CANVAS.left_top() + Vec2::splat(GAP);
    app.panel_positions.variables = Some(pos2(0.5, 0.5));
    assert_eq!(panel_layout::origin(&app, Panel::Settings, CANVAS), corner);
    let dims = panel_layout::origin(&app, Panel::Dimensions, CANVAS);
    assert_eq!(dims, corner + Vec2::new(280.0 + GAP, 0.0));
    assert_eq!(
        panel_layout::origin(&app, Panel::Variables, CANVAS),
        CANVAS.center(),
        "a dragged panel sits at its own place"
    );

    app.show_settings_panel = false;
    assert_eq!(
        panel_layout::origin(&app, Panel::Dimensions, CANVAS),
        corner
    );
}

/// Radius of a grip dot at `Sm` size (1.9 of 24 grid units).
const GRIP_DOT_R: f32 = 1.9 * 14.0 / 24.0;

struct Harness {
    ctx: egui::Context,
    time: f64,
}

impl Harness {
    /// Runs one frame of the Settings panel with `events`; the centers of the
    /// grip's dots.
    fn frame(&mut self, app: &mut OctantApp, events: Vec<Event>) -> Vec<Pos2> {
        self.time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_max(Pos2::ZERO, CANVAS.max)),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self
            .ctx
            .run_ui(input, |ui| show_settings_window(app, ui.ctx(), CANVAS));
        output.textures_delta.clear();
        output
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Circle(c) if (c.radius - GRIP_DOT_R).abs() < 0.01 => Some(c.center),
                _ => None,
            })
            .collect()
    }

    fn panel(&self) -> Rect {
        self.ctx
            .memory(|m| m.area_rect(Id::new("octant_settings_area")))
            .expect("the settings panel is shown")
    }

    fn pointer(&mut self, app: &mut OctantApp, pos: Pos2, pressed: Option<bool>) {
        let mut events = vec![Event::PointerMoved(pos)];
        if let Some(pressed) = pressed {
            events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            });
        }
        self.frame(app, events);
    }
}

#[test]
fn the_grip_drags_the_settings_panel_and_double_click_docks_it() {
    let mut app = OctantApp {
        show_settings_panel: true,
        ..Default::default()
    };
    let mut h = Harness {
        ctx: egui::Context::default(),
        time: 0.0,
    };
    let mut dots = Vec::new();
    for _ in 0..3 {
        dots = h.frame(&mut app, Vec::new());
    }
    assert_eq!(dots.len(), 6, "the grip is always shown");
    let grip = dots.iter().fold(Vec2::ZERO, |sum, p| sum + p.to_vec2()) / 6.0;
    let grip = grip.to_pos2();
    let docked = h.panel();

    h.pointer(&mut app, grip, None);
    h.pointer(&mut app, grip, Some(true));
    let delta = Vec2::new(300.0, 200.0);
    for step in 1..=10 {
        h.pointer(&mut app, grip + delta * (step as f32 / 10.0), None);
    }
    h.pointer(&mut app, grip + delta, Some(false));
    h.frame(&mut app, Vec::new());

    assert!(app.panel_positions.settings.is_some());
    let moved = h.panel().min - docked.min;
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let grip = grip + moved;
    for _ in 0..2 {
        h.pointer(&mut app, grip, Some(true));
        h.pointer(&mut app, grip, Some(false));
    }
    h.frame(&mut app, Vec::new());
    assert_eq!(app.panel_positions.settings, None);
    assert_eq!(h.panel().min, docked.min);
}

#[test]
fn panels_draw_over_colorbars_even_after_a_colorbar_click() {
    let mut app = OctantApp {
        show_settings_panel: true,
        show_colorbar: true,
        ..Default::default()
    };
    app.layers.base.colorbar.pos = Some(crate::ui::drag_grip::to_fraction(
        pos2(300.0, 200.0),
        CANVAS,
    ));
    let ctx = egui::Context::default();
    let mut time = 0.0;
    let mut frame = |app: &mut OctantApp, events: Vec<Event>| {
        time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_max(Pos2::ZERO, CANVAS.max)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            crate::ui::colorbar::show_colorbar_overlay(app, ui.ctx(), CANVAS);
            show_settings_window(app, ui.ctx(), CANVAS);
        });
        output.textures_delta.clear();
    };
    for _ in 0..3 {
        frame(&mut app, Vec::new());
    }
    let colorbar_id = Id::new(("octant_colorbar_overlay", crate::app::layers::LayerId::BASE));
    let area = |id: Id| ctx.memory(|m| m.area_rect(id)).expect("shown");
    let colorbar = area(colorbar_id);
    let settings = area(Id::new("octant_settings_area"));
    let overlap = colorbar.intersect(settings);
    assert!(overlap.is_positive(), "{colorbar:?} vs {settings:?}");

    // Click the colorbar where only it lies, which raises it in its order.
    let only_bar = pos2(colorbar.right() - 30.0, colorbar.center().y);
    assert!(!settings.contains(only_bar));
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: only_bar,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&mut app, vec![Event::PointerMoved(only_bar), event]);
    }
    let top = ctx.layer_id_at(overlap.center()).map(|l| l.id);
    assert_eq!(top, Some(Id::new("octant_settings_area")));
}

#[test]
fn the_catalog_backdrop_covers_colorbars() {
    let mut app = OctantApp {
        show_colorbar: true,
        ..Default::default()
    };
    let ctx = egui::Context::default();
    let mut time = 0.0;
    let mut frame = |app: &mut OctantApp, events: Vec<Event>| {
        time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_max(Pos2::ZERO, CANVAS.max)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            crate::ui::colorbar::show_colorbar_overlay(app, ui.ctx(), CANVAS);
            crate::ui::catalog::show_catalog_window(app, ui.ctx());
        });
        output.textures_delta.clear();
    };
    for _ in 0..3 {
        frame(&mut app, Vec::new());
    }
    let colorbar_id = Id::new(("octant_colorbar_overlay", crate::app::layers::LayerId::BASE));
    let colorbar = ctx.memory(|m| m.area_rect(colorbar_id)).expect("shown");
    // Open and close the catalog, then click the colorbar, raising it.
    app.show_catalog_window = true;
    frame(&mut app, Vec::new());
    app.show_catalog_window = false;
    for _ in 0..2 {
        frame(&mut app, Vec::new());
    }
    let at = colorbar.center();
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&mut app, vec![Event::PointerMoved(at), event]);
    }
    app.show_catalog_window = true;
    for _ in 0..3 {
        frame(&mut app, Vec::new());
    }
    let top = ctx.layer_id_at(at).map(|l| l.id);
    assert_eq!(top, Some(Id::new("catalog_modal_backdrop")));
}
