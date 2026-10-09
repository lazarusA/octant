//! Interactive Drag handles and min/max clip triangle widgets.

use super::axis::BarAxis;
use super::ticks::format_scientific_tick;
use crate::app::layers::{BarOrientation, ColorStyle, LayerId};
use crate::ui::color_picker::{ColorShape, ShapeColorPicker};
use egui::{Pos2, Rect, Ui, Vec2};

/// Size of a range input box.
const INPUT: Vec2 = Vec2::new(60.0, 18.0);
/// Length of a clip triangle along the bar.
const TRI: f32 = 12.0;
/// Gap between a vertical bar's end and its range input, past the triangle.
const VERTICAL_END_GAP: f32 = TRI + 4.0;
/// Room a vertical bar needs at each end for its triangle and range input.
pub const VERTICAL_END_ROOM: f32 = VERTICAL_END_GAP + INPUT.y + 2.0;

/// The centers of the min and max range inputs: below the ends of a
/// horizontal bar, below and above a vertical one.
fn input_centers(axis: BarAxis) -> [Pos2; 2] {
    let r = axis.rect;
    match axis.orientation {
        BarOrientation::Horizontal => {
            let y = r.max.y + 6.0 + INPUT.y / 2.0;
            [Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)]
        }
        BarOrientation::Vertical => {
            let off = VERTICAL_END_GAP + INPUT.y / 2.0;
            let x = r.center().x;
            [Pos2::new(x, r.max.y + off), Pos2::new(x, r.min.y - off)]
        }
    }
}

/// Renders min/max drag value input boxes at the colorbar ends, editing
/// `color`'s range (and locking it).
pub fn draw_end_range_inputs(ui: &mut Ui, axis: BarAxis, color: &mut ColorStyle) {
    let (min_val, max_val) = (color.range_min, color.range_max);
    let drag_speed = ((max_val - min_val).abs() / 100.0).max(1e-4);
    let [min_center, max_center] = input_centers(axis);

    let mut new_min = color.range_min;
    let mut new_max = color.range_max;
    let input = |ui: &mut Ui, center: Pos2, value: &mut f32, hover: &str| {
        ui.put(
            Rect::from_center_size(center, INPUT),
            egui::DragValue::new(value)
                .speed(drag_speed)
                .custom_formatter(|val, _| format_scientific_tick(val as f32))
                .custom_parser(|s| s.trim().parse::<f64>().ok()),
        )
        .on_hover_text(hover)
    };
    let min_resp = input(
        ui,
        min_center,
        &mut new_min,
        "Lower end range (Min). Drag to adjust or click to type.",
    );
    let max_resp = input(
        ui,
        max_center,
        &mut new_max,
        "Upper end range (Max). Drag to adjust or click to type.",
    );

    if min_resp.changed() || new_min != color.range_min {
        color.range_min = new_min;
        color.lock_bounds = true;
    }
    if max_resp.changed() || new_max != color.range_max {
        color.range_max = new_max;
        color.lock_bounds = true;
    }
}

/// The low and high clip triangles' rects and shapes, past the bar's ends.
fn triangles(axis: BarAxis) -> [(Rect, ColorShape); 2] {
    let r = axis.rect;
    match axis.orientation {
        BarOrientation::Horizontal => [
            (
                Rect::from_min_max(Pos2::new(r.min.x - TRI, r.min.y), r.left_bottom()),
                ColorShape::LeftTriangle,
            ),
            (
                Rect::from_min_max(r.right_top(), Pos2::new(r.max.x + TRI, r.max.y)),
                ColorShape::RightTriangle,
            ),
        ],
        BarOrientation::Vertical => [
            (
                Rect::from_min_max(r.left_bottom(), Pos2::new(r.max.x, r.max.y + TRI)),
                ColorShape::DownTriangle,
            ),
            (
                Rect::from_min_max(Pos2::new(r.min.x, r.min.y - TRI), r.right_top()),
                ColorShape::UpTriangle,
            ),
        ],
    }
}

/// Renders `color`'s low-clip and high-clip triangles on the ends of layer
/// `id`'s colorbar when enabled.
pub fn draw_clip_triangles(ui: &mut Ui, axis: BarAxis, color: &mut ColorStyle, id: LayerId) {
    let [(low_rect, low_shape), (high_rect, high_shape)] = triangles(axis);

    if color.use_lowclip {
        ShapeColorPicker::new(
            ("colorbar_lowclip_picker", id),
            &mut color.lowclip_color,
            low_shape,
        )
        .title("Low Clip Color (< Min)")
        .tooltip("Low Clip color (< Min). Click to select color.")
        .show_at(ui, low_rect);
    }

    if color.use_highclip {
        ShapeColorPicker::new(
            ("colorbar_highclip_picker", id),
            &mut color.highclip_color,
            high_shape,
        )
        .title("High Clip Color (> Max)")
        .tooltip("High Clip color (> Max). Click to select color.")
        .anchor_offset(egui::Vec2::new(-170.0, -250.0))
        .show_at(ui, high_rect);
    }
}
