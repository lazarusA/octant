//! Rows keep their items' reading order when they stack, and a labeled
//! slider learns its value box so a wide value stays within the row and
//! never moves back beside its label.

use super::{SliderLabel, labeled_slider, right_aligned};
use crate::ui::number_input::LocalizedInput;
use crate::ui::test_input::Harness;
use egui::{Rect, Ui, pos2, vec2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

/// Runs `add` in a `width` wide column for a few frames.
fn in_column(width: f32, mut add: impl FnMut(&mut Ui)) {
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut |ui: &mut Ui| {
        ui.allocate_ui(vec2(width, 400.0), |ui| {
            ui.set_width(width);
            add(ui);
        });
    });
}

/// The rects of two `width` wide items in a `column` wide right-aligned row.
fn two_items(column: f32, width: f32) -> [Rect; 2] {
    let mut rects = [Rect::NOTHING; 2];
    in_column(column, |ui| {
        right_aligned(ui, &[width, width], |ui, item| {
            rects[item] = ui
                .allocate_exact_size(vec2(width, 18.0), egui::Sense::hover())
                .0;
        });
    });
    rects
}

#[test]
fn items_fit_on_one_row_with_item_0_rightmost() {
    let [first, second] = two_items(300.0, 100.0);
    assert_eq!(first.top(), second.top(), "one row");
    assert!(first.left() > second.right(), "item 0 is the rightmost");
}

#[test]
fn stacked_items_keep_their_reading_order() {
    let [first, second] = two_items(150.0, 100.0);
    assert!(first.top() > second.bottom(), "item 0 goes below item 1");
    assert!(
        (first.right() - second.right()).abs() < 0.5,
        "both right-aligned"
    );
}

#[test]
fn a_wide_value_box_is_learned_and_stays_in_the_row() {
    let column = 160.0;
    let mut value = 123_456.75_f64;
    let mut right = 0.0;
    in_column(column, |ui| {
        let slider = egui::Slider::new(&mut value, 0.0..=1_000_000.0)
            .localized()
            .suffix(" units");
        let label = SliderLabel::After("A rather long label");
        right = labeled_slider(ui, "wide", label, slider).rect.right();
    });
    assert!(
        right <= column + 0.5,
        "slider ends at {right}, past {column}"
    );
}

#[test]
fn a_slider_under_its_label_stays_there_when_its_value_narrows() {
    let column = 220.0;
    let mut h = Harness::new(SCREEN);
    let mut value = 999_999.5_f64;
    let mut bottoms = Vec::new();
    for narrow in [false, false, false, true, true, true] {
        if narrow {
            value = 1.0;
        }
        h.frame(Vec::new(), &mut |ui: &mut Ui| {
            ui.allocate_ui(vec2(column, 400.0), |ui| {
                ui.set_width(column);
                let slider = egui::Slider::new(&mut value, 0.0..=1_000_000.0)
                    .localized()
                    .suffix(" units");
                let label = SliderLabel::After("Label");
                bottoms.push(labeled_slider(ui, "flip", label, slider).rect.bottom());
            });
        });
    }
    let stacked = bottoms[2];
    assert!(
        stacked > bottoms[0],
        "a wide value moves the slider under its label"
    );
    assert!(
        bottoms[3..]
            .iter()
            .all(|&bottom| (bottom - stacked).abs() < 0.5),
        "it stays there: {bottoms:?}"
    );
}

/// The response of a slider labeled "Label" over `value` in a `column` wide
/// column, one frame.
fn slider_frame(h: &mut Harness, column: f32, value: &mut f64) -> egui::Response {
    let mut response = None;
    h.frame(Vec::new(), &mut |ui: &mut Ui| {
        ui.allocate_ui(vec2(column, 400.0), |ui| {
            ui.set_width(column);
            let slider = egui::Slider::new(&mut *value, 0.0..=1_000_000.0)
                .localized()
                .suffix(" units");
            let label = SliderLabel::After("Label");
            response = Some(labeled_slider(ui, "kept", label, slider));
        });
    });
    response.expect("the slider is drawn")
}

#[test]
fn a_slider_keeps_its_id_when_it_moves_under_its_label() {
    let mut h = Harness::new(SCREEN);
    let mut value = 1.0;
    let beside = slider_frame(&mut h, 400.0, &mut value);
    let under = slider_frame(&mut h, 120.0, &mut value);
    assert!(
        under.rect.top() > beside.rect.top(),
        "the slider moved under its label"
    );
    assert_eq!(beside.id, under.id, "a drag or edit carries on");
}

#[test]
fn a_kept_value_box_is_measured_again_when_the_style_changes() {
    let mut h = Harness::new(SCREEN);
    let mut value = 999_999.5;
    let column = 220.0;
    let wide: Vec<f32> = (0..3)
        .map(|_| slider_frame(&mut h, column, &mut value).rect.bottom())
        .collect();
    assert!(
        wide[2] > wide[0],
        "a wide value moves the slider under its label"
    );
    value = 1.0;
    h.ctx
        .all_styles_mut(|style| style.spacing.item_spacing.x += 0.5);
    let narrow: Vec<f32> = (0..3)
        .map(|_| slider_frame(&mut h, column, &mut value).rect.bottom())
        .collect();
    assert!(
        (narrow[2] - wide[0]).abs() < 0.5,
        "back beside its label: {wide:?} then {narrow:?}"
    );
}
