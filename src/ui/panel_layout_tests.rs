//! Docked panels line up and close up when one is dragged away; the Settings
//! panel's grip drags it and a double-click docks it again.

use crate::app::OctantApp;
use crate::ui::panel_layout::{self, GAP, Panel};
use crate::ui::settings::show_settings_window;
use crate::ui::test_input::Harness;
use egui::{Id, Pos2, Rect, Ui, Vec2, pos2};

const CANVAS: Rect = Rect::from_min_max(pos2(40.0, 30.0), pos2(1240.0, 930.0));

fn app() -> OctantApp {
    let mut app = OctantApp::default();
    app.layout.show_variables_overlay = true;
    app.layout.show_settings_panel = true;
    app.layout.variables_overlay_width = 300.0;
    app.layout.settings_overlay_width = 280.0;
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
    app.layout.panel_positions.variables = Some(pos2(0.5, 0.5));
    assert_eq!(panel_layout::origin(&app, Panel::Settings, CANVAS), corner);
    let dims = panel_layout::origin(&app, Panel::Dimensions, CANVAS);
    assert_eq!(dims, corner + Vec2::new(280.0 + GAP, 0.0));
    assert_eq!(
        panel_layout::origin(&app, Panel::Variables, CANVAS),
        CANVAS.center(),
        "a dragged panel sits at its own place"
    );

    app.layout.show_settings_panel = false;
    assert_eq!(
        panel_layout::origin(&app, Panel::Dimensions, CANVAS),
        corner
    );
}

/// Radius of a grip dot at `Sm` size (1.9 of 24 grid units).
const GRIP_DOT_R: f32 = 1.9 * 14.0 / 24.0;

/// One frame of `app`'s Settings panel.
fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_settings_window(app, ui.ctx(), CANVAS)
}

#[test]
fn the_grip_drags_the_settings_panel_and_double_click_docks_it() {
    let mut app = OctantApp::default();
    app.layout.show_settings_panel = true;
    let mut h = Harness::new(Rect::from_min_max(Pos2::ZERO, CANVAS.max));
    let shapes = h.settle(3, &mut run(&mut app));
    let dots: Vec<Pos2> = shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Circle(c) if (c.radius - GRIP_DOT_R).abs() < 0.01 => Some(c.center),
            _ => None,
        })
        .collect();
    assert_eq!(dots.len(), 6, "the grip is always shown");
    let grip = (dots.iter().fold(Vec2::ZERO, |sum, p| sum + p.to_vec2()) / 6.0).to_pos2();
    let panel = |h: &Harness| h.area(Id::new("octant_settings_area"));
    let docked = panel(&h);

    let delta = Vec2::new(300.0, 200.0);
    h.drag(grip, delta, &mut run(&mut app));
    assert!(app.layout.panel_positions.settings.is_some());
    let moved = panel(&h).min - docked.min;
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let grip = grip + moved;
    h.click(grip, &mut run(&mut app));
    h.click(grip, &mut run(&mut app));
    h.frame(Vec::new(), &mut run(&mut app));
    assert_eq!(app.layout.panel_positions.settings, None);
    assert_eq!(panel(&h).min, docked.min);
}
