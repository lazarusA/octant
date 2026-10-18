//! The colorbar controls driven headlessly: shown only while hovered (or
//! the Color window is open) and never during export; the cog opens and
//! closes the Color window, the grip drags and resets the panel, the flip
//! turns it, and a click on the bar opens the colormap picker.

use super::show_colorbar_overlay;
use super::{color_menu, controls};
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::ui::test_input::Harness;
use egui::epaint::ClippedShape;
use egui::{Rect, Ui, Vec2, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

/// One frame of `app`'s colorbars.
fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_colorbar_overlay(app, ui.ctx(), SCREEN)
}

/// A harness that has laid the colorbar out over a few frames.
fn harness(app: &mut OctantApp) -> Harness {
    let mut h = Harness::new(SCREEN);
    h.settle(3, &mut run(app));
    h
}

/// The base layer's colorbar panel.
fn panel(h: &Harness) -> Rect {
    h.area(egui::Id::unique(("octant_colorbar_overlay", LayerId::BASE)))
}

/// How many circles `shapes` holds (the grip's dots).
fn circles(shapes: &[ClippedShape]) -> usize {
    shapes
        .iter()
        .filter(|c| matches!(c.shape, egui::Shape::Circle(_)))
        .count()
}

#[test]
fn controls_show_only_while_the_panel_is_hovered() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    let away = h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    assert_eq!(circles(&away), 0);
    let inside = h.pointer(panel(&h).center(), None, &mut run(&mut app));
    assert!(circles(&inside) >= 6);
}

#[test]
fn controls_hide_during_export() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    app.pending_export = Some(crate::export::PendingExportRequest {
        format: Default::default(),
        target: Default::default(),
        roi: Default::default(),
        jpeg_quality: 90,
        copy_to_clipboard: false,
        output_path: None,
        canvas_rect_in_points: Rect::NOTHING,
        pixels_per_point: 1.0,
        screenshot: Default::default(),
    });
    let inside = h.pointer(panel(&h).center(), None, &mut run(&mut app));
    assert_eq!(circles(&inside), 0);
}

#[test]
fn dragging_the_grip_moves_the_panel_and_double_click_resets_it() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    let before = panel(&h);
    let [grip, _] = controls::rects(before);
    let delta = Vec2::new(-200.0, -300.0);
    h.drag(grip.center(), delta, &mut run(&mut app));

    assert!(app.layers.base.colorbar.pos.is_some());
    let moved = panel(&h).center_bottom() - before.center_bottom();
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let [grip, _] = controls::rects(panel(&h));
    h.click(grip.center(), &mut run(&mut app));
    h.click(grip.center(), &mut run(&mut app));
    assert_eq!(app.layers.base.colorbar.pos, None);
}

#[test]
fn the_flip_turns_the_bar() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    let [_, flip] = controls::rects(panel(&h));
    h.click(flip.center(), &mut run(&mut app));
    assert_eq!(
        app.layers.base.colorbar.orientation,
        BarOrientation::Vertical
    );
    h.settle(3, &mut run(&mut app));
    let panel = panel(&h);
    assert!(panel.height() > panel.width(), "vertical panel: {panel:?}");
    assert!(SCREEN.contains_rect(panel));
}

/// The bar of panel `panel` in `shapes`: the mesh of many vertices inside it.
fn bar(shapes: &[ClippedShape], panel: Rect) -> Rect {
    shapes
        .iter()
        .find_map(|c| match &c.shape {
            egui::Shape::Mesh(mesh) if mesh.vertices.len() >= 256 => {
                let points: Vec<_> = mesh.vertices.iter().map(|v| v.pos).collect();
                Some(Rect::from_points(&points)).filter(|r| panel.contains_rect(*r))
            }
            _ => None,
        })
        .expect("the panel paints its bar")
}

#[test]
fn the_cog_opens_the_color_window_and_stays_while_it_is_open() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    let menu = color_menu::area_id(LayerId::BASE);
    h.pointer(panel(&h).center(), None, &mut run(&mut app));
    let cog = controls::cog_rect(panel(&h));
    h.click(cog.center(), &mut run(&mut app));
    assert!(color_menu::is_open(&h.ctx, LayerId::BASE));
    h.settle(3, &mut run(&mut app));
    let window = h.area(menu);
    assert!(window.is_positive(), "{window:?}");
    assert!(
        window.max.y <= panel(&h).min.y,
        "above the bottom bar: {window:?}"
    );

    // Away from the panel the cog stays, so a click closes the window.
    h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    h.click(cog.center(), &mut run(&mut app));
    assert!(!color_menu::is_open(&h.ctx, LayerId::BASE));
}

#[test]
fn clicking_the_bar_opens_the_picker_on_its_layer() {
    let mut app = OctantApp::default();
    let overlay = app.layers.push(crate::app::layers::Source::default());
    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(3, &mut run(&mut app));
    let panel = h.area(egui::Id::unique(("octant_colorbar_overlay", overlay)));
    let bar = bar(&shapes, panel);
    h.click(bar.center(), &mut run(&mut app));
    assert_eq!(app.colormaps.target, Some(overlay));
    let popup = egui::Id::unique(("colorbar_bar", overlay)).with("popup");
    assert!(egui::Popup::is_id_open(&h.ctx, popup));
}

#[test]
fn a_hidden_colorbar_is_not_drawn() {
    let mut app = OctantApp::default();
    app.layers.base.colorbar.shown = false;
    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(3, &mut run(&mut app));
    assert!(
        !shapes
            .iter()
            .any(|c| matches!(&c.shape, egui::Shape::Mesh(m) if m.vertices.len() >= 256))
    );
}

#[test]
fn the_close_button_in_the_corner_closes_the_color_window() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    color_menu::set_open(&h.ctx, LayerId::BASE, true);
    h.settle(3, &mut run(&mut app));
    let window = h.area(color_menu::area_id(LayerId::BASE));
    let margin = egui::Frame::popup(&egui::Style::default()).total_margin();
    let side = crate::ui::icons::ToolbarButton::side(crate::ui::icons::IconSize::Sm);
    let close = pos2(
        window.max.x - margin.right - side / 2.0,
        window.min.y + margin.top + side / 2.0,
    );
    h.click(close, &mut run(&mut app));
    assert!(!color_menu::is_open(&h.ctx, LayerId::BASE));
}

#[test]
fn a_tap_reveals_the_controls_so_a_second_tap_opens_the_color_window() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    let cog = controls::cog_rect(panel(&h));
    // On a touch screen the first tap only reveals the controls.
    h.tap(cog.center(), &mut run(&mut app));
    assert!(!color_menu::is_open(&h.ctx, LayerId::BASE));
    h.tap(cog.center(), &mut run(&mut app));
    assert!(color_menu::is_open(&h.ctx, LayerId::BASE));
}

#[test]
fn a_tap_elsewhere_hides_the_revealed_controls() {
    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    h.tap(panel(&h).center(), &mut run(&mut app));
    let revealed = h.settle(1, &mut run(&mut app));
    assert!(circles(&revealed) >= 6, "the grip shows after a tap");
    h.tap(pos2(5.0, 5.0), &mut run(&mut app));
    let hidden = h.settle(1, &mut run(&mut app));
    assert_eq!(circles(&hidden), 0);
}

#[test]
fn hovering_the_bar_outlines_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    // Past the panel's fade-in, so its strokes are painted.
    let shapes = h.settle(30, &mut run(&mut app));
    let bar = bar(&shapes, panel(&h));
    let stroke = h
        .ctx
        .style_of(h.ctx.theme())
        .visuals
        .widgets
        .hovered
        .fg_stroke;
    let outlined = |shapes: &[ClippedShape]| {
        shapes.iter().any(|c| match &c.shape {
            egui::Shape::Rect(r) => r.stroke == stroke && r.rect.contains_rect(bar),
            _ => false,
        })
    };
    let away = h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    assert!(!outlined(&away));
    let over = h.pointer(bar.center(), None, &mut run(&mut app));
    assert!(outlined(&over));
}

#[test]
fn keyboard_focus_on_the_bar_reveals_the_controls_and_picks_a_colormap() {
    use crate::ui::keyboard::focus;
    use egui::{Key, Modifiers};

    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    focus::request(&h.ctx, super::bar_id(LayerId::BASE));
    let shapes = h.settle(2, &mut run(&mut app));
    assert!(circles(&shapes) >= 6, "focus in the panel shows the grip");

    let press = |h: &mut Harness, app: &mut OctantApp, key| {
        h.key(key, Modifiers::NONE, &mut run(app));
        h.settle(2, &mut run(app));
    };
    press(&mut h, &mut app, Key::Enter);
    assert!(egui::Popup::is_any_open(&h.ctx), "Enter opens the picker");
    // The search field has focus: Down enters the list, Down moves on.
    press(&mut h, &mut app, Key::ArrowDown);
    press(&mut h, &mut app, Key::ArrowDown);
    let picked = app.colormaps.picker.focused_row.expect("a row is focused");
    press(&mut h, &mut app, Key::Enter);
    assert_eq!(app.layers.base.color.colormap, picked);
}
