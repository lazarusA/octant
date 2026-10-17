//! A dimension's index range: egui's two-handled `RangeSlider`, its numbers
//! in fixed-width boxes on both sides.

use crate::ui::flow_row::{self, VALUE_BOX_W};
use crate::ui::number_input::LocalizedInput;
use egui::{RangeSlider, Ui, style::HandleShape, vec2};

/// Thickness of the track and its selected range.
const RAIL_H: f32 = 4.0;
/// Least row height: egui sizes the round handles from it (a 2.5th of it
/// is their radius), so 15 keeps them near 6 px where the text allows.
const ROW_H: f32 = 15.0;

/// Range slider with numeric input fields on both sides.
///
/// Laid out left to right (start box, track, end box): both boxes are
/// `VALUE_BOX_W` wide and the track fills the space between, so the tracks
/// of every dimension row line up and never resize while a value changes.
pub fn double_slider_with_inputs(
    ui: &mut Ui,
    id_source: impl egui::AsIdSalt,
    start: &mut usize,
    end: &mut usize,
    min: usize,
    max: usize,
) -> bool {
    let changed = ui
        .push_id(id_source, |ui| {
            // Room left beside both boxes and the gaps after them.
            let boxes = 2.0 * VALUE_BOX_W + ui.spacing().item_spacing.x;
            let track = flow_row::track_width(ui, boxes);
            let spacing = ui.spacing_mut();
            spacing.interact_size = vec2(VALUE_BOX_W, ROW_H);
            spacing.slider_width = track;
            spacing.slider_rail_height = RAIL_H;
            ui.add(
                RangeSlider::new(start, end, min..=max)
                    .handle_shape(HandleShape::Circle)
                    .drag_value_speed(1.0)
                    .localized(),
            )
        })
        .inner
        .changed();

    *start = (*start).clamp(min, max);
    *end = (*end).clamp(min, max).max(*start);

    changed
}
