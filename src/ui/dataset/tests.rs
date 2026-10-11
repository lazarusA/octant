use super::show_dataset_window;
use crate::app::OctantApp;
use crate::ui::panel_layout::{self, Panel};
use crate::ui::test_input::Harness;
use egui::{Id, Pos2, Rect, Ui, Vec2, pos2};

const CANVAS: Rect = Rect::from_min_max(pos2(40.0, 30.0), pos2(1240.0, 930.0));
const GRIP_DOT_R: f32 = 1.9 * 14.0 / 24.0;

fn app() -> OctantApp {
    let mut app = OctantApp::default();
    app.layout.show_dataset_panel = true;
    app
}

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_dataset_window(app, ui.ctx(), CANVAS)
}

#[test]
fn dataset_panel_renders_at_docked_origin_when_open() {
    let mut app = app();
    let mut h = Harness::new(CANVAS);
    h.settle(3, &mut run(&mut app));

    let panel_rect = h.area(Id::new("octant_dataset_area"));
    let expected_origin = panel_layout::origin(&app, Panel::Dataset, CANVAS);
    assert_eq!(panel_rect.min, expected_origin);
    assert!(app.layout.dataset_overlay_width > 0.0);
}

#[test]
fn dataset_panel_hidden_when_flag_is_false() {
    let mut app = app();
    app.layout.show_dataset_panel = false;
    let mut h = Harness::new(CANVAS);
    h.settle(3, &mut run(&mut app));

    assert_eq!(app.layout.dataset_overlay_width, 0.0);
    assert!(
        h.ctx
            .memory(|m| m.area_rect(Id::new("octant_dataset_area")))
            .is_none()
    );
}

#[test]
fn grip_drags_dataset_panel_and_double_click_docks_it() {
    let mut app = app();
    let mut h = Harness::new(CANVAS);
    let shapes = h.settle(3, &mut run(&mut app));

    let dots: Vec<Pos2> = shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Circle(c) if (c.radius - GRIP_DOT_R).abs() < 0.01 => Some(c.center),
            _ => None,
        })
        .collect();
    assert_eq!(dots.len(), 6, "the dataset panel grip is shown");
    let grip = (dots.iter().fold(Vec2::ZERO, |sum, p| sum + p.to_vec2()) / 6.0).to_pos2();
    let panel = |h: &Harness| h.area(Id::new("octant_dataset_area"));
    let docked = panel(&h);

    let delta = Vec2::new(240.0, 160.0);
    h.drag(grip, delta, &mut run(&mut app));
    assert!(app.layout.panel_positions.dataset.is_some());
    let moved = panel(&h).min - docked.min;
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let grip = grip + moved;
    h.click(grip, &mut run(&mut app));
    h.click(grip, &mut run(&mut app));
    h.frame(Vec::new(), &mut run(&mut app));
    assert_eq!(app.layout.panel_positions.dataset, None);
    assert_eq!(panel(&h).min, docked.min);
}
