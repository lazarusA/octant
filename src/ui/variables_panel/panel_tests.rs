//! The Dimensions panel stays inside the canvas: its sliders scroll on a
//! short canvas, and a variable's attributes open in a popup instead of
//! stretching the panel. Unticking a dimension clears its roles.

use super::{show_variable_controls, show_variable_info};
use crate::app::test_support::{memory_app, selection_of};
use crate::data::VariableInfo;
use crate::ui::panel_layout::{self, Panel};
use crate::ui::test_input::Harness;
use egui::epaint::ClippedShape;
use egui::{Pos2, Rect, Ui, pos2};
use std::cell::Cell;
use std::collections::HashMap;

/// Too short for the panel's header, Plot Data and every slider at once.
const SHORT: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 320.0));
const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));
/// Rounding of measured against drawn positions.
const SLACK: f32 = 1.0;

/// The center of the painted text `label` in `shapes`, if any.
fn text_center(shapes: &[ClippedShape], label: &str) -> Option<Pos2> {
    text_rect(shapes, label).map(|r| r.center())
}

#[test]
fn sliders_scroll_on_a_short_canvas_and_the_panel_stays_docked() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.layout.show_variable_controls = true;
    let origin = panel_layout::origin(&app, Panel::Dimensions, SHORT);

    let mut h = Harness::new(SHORT);
    h.settle(8, &mut |ui: &mut Ui| {
        show_variable_controls(&mut app, ui.ctx(), SHORT)
    });
    let area = h.area(Panel::Dimensions.area_id());

    assert!(
        area.bottom() <= SHORT.bottom() + SLACK,
        "past the canvas: {area:?}"
    );
    assert!(
        (area.top() - origin.y).abs() <= SLACK,
        "pushed off its place: {area:?}"
    );
}

#[test]
fn attributes_open_in_a_popup_and_leave_the_card_height() {
    let history = "regridded, then averaged over ensemble members; ".repeat(12);
    let info = VariableInfo {
        name: "t2m".into(),
        shape: vec![3, 5, 4],
        dimension_names: vec!["time".into(), "lat".into(), "lon".into()],
        attributes: HashMap::from([("history".to_string(), history.clone())]),
        ..Default::default()
    };
    let height = Cell::new(0.0);
    let mut run = |ui: &mut Ui| {
        let card = ui.scope(|ui| show_variable_info(ui, &info)).response.rect;
        height.set(card.height());
    };

    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(3, &mut run);
    let closed = height.get();
    assert!(
        text_center(&shapes, &history).is_none(),
        "hidden until asked"
    );
    let button = text_center(&shapes, "Attributes").expect("the button is shown");

    h.click(button, &mut run);
    let shapes = h.settle(3, &mut run);
    assert!(egui::Popup::is_any_open(&h.ctx), "the popup opens");
    assert!(
        text_center(&shapes, &history).is_some(),
        "the value is shown"
    );
    assert_eq!(height.get(), closed, "the card keeps its height");
}

#[test]
fn unticking_a_dimension_clears_its_roles() {
    use crate::app::{AnimationRole, SpatialRole};
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.layout.show_variable_controls = true;
    let lon = 2;
    assert_ne!(app.selected.dim_config[lon].spatial, SpatialRole::None);
    let mut run = |ui: &mut Ui| show_variable_controls(&mut app, ui.ctx(), SCREEN);

    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(3, &mut run);
    // The checkbox sits one item spacing left of the dimension's name.
    let name = text_rect(&shapes, "lon").expect("the lon row is shown");
    let spacing = egui::Style::default().spacing;
    let checkbox = pos2(
        name.left() - spacing.item_spacing.x - spacing.interact_size.y * 0.5,
        name.center().y,
    );
    h.click(checkbox, &mut run);
    h.settle(3, &mut run);

    let c = app.selected.dim_config[lon];
    assert!(!c.active, "stays unticked");
    assert_eq!(
        (c.spatial, c.animation),
        (SpatialRole::None, AnimationRole::None)
    );
}

/// The painted rect of the text `label` in `shapes`, if any.
fn text_rect(shapes: &[ClippedShape], label: &str) -> Option<Rect> {
    shapes.iter().find_map(|clipped| match &clipped.shape {
        egui::Shape::Text(text) if text.galley.text() == label => {
            Some(Rect::from_min_size(text.pos, text.galley.size()))
        }
        _ => None,
    })
}
