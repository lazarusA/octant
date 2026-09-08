//! Interactive Drag handles and min/max clip triangle widgets.

use super::ticks::format_scientific_tick;
use crate::app::OctantApp;
use egui::{Pos2, Rect, Ui, Vec2};

/// Renders min/max drag value input boxes centered directly under the colorbar ends.
pub fn draw_end_range_inputs(
    app: &mut OctantApp,
    ui: &mut Ui,
    bar_rect: Rect,
    min_val: f32,
    max_val: f32,
) {
    let input_w = 60.0;
    let input_h = 18.0;
    let drag_speed = ((max_val - min_val).abs() / 100.0).max(1e-4);

    let mut new_min = app.color_range_min;
    let mut new_max = app.color_range_max;

    let min_rect = Rect::from_center_size(
        Pos2::new(bar_rect.min.x, bar_rect.max.y + 6.0 + input_h / 2.0),
        Vec2::new(input_w, input_h),
    );
    let max_rect = Rect::from_center_size(
        Pos2::new(bar_rect.max.x, bar_rect.max.y + 6.0 + input_h / 2.0),
        Vec2::new(input_w, input_h),
    );

    let min_resp = ui
        .put(
            min_rect,
            egui::DragValue::new(&mut new_min)
                .speed(drag_speed)
                .custom_formatter(|val, _| format_scientific_tick(val as f32))
                .custom_parser(|s| s.trim().parse::<f64>().ok()),
        )
        .on_hover_text("Lower end range (Min). Drag to adjust or click to type.");

    let max_resp = ui
        .put(
            max_rect,
            egui::DragValue::new(&mut new_max)
                .speed(drag_speed)
                .custom_formatter(|val, _| format_scientific_tick(val as f32))
                .custom_parser(|s| s.trim().parse::<f64>().ok()),
        )
        .on_hover_text("Upper end range (Max). Drag to adjust or click to type.");

    if min_resp.changed() || new_min != app.color_range_min {
        app.color_range_min = new_min;
        app.volume_cmin = new_min;
        app.lock_color_bounds = true;
    }
    if max_resp.changed() || new_max != app.color_range_max {
        app.color_range_max = new_max;
        app.volume_cmax = new_max;
        app.lock_color_bounds = true;
    }
}

/// Renders low-clip and high-clip colored triangle widgets on colorbar ends when enabled.
pub fn draw_clip_triangles(app: &mut OctantApp, ui: &mut Ui, bar_rect: Rect) {
    let tri_w = 12.0_f32;

    if app.use_lowclip {
        let low_tri_rect = Rect::from_min_max(
            Pos2::new(bar_rect.min.x - tri_w, bar_rect.min.y),
            Pos2::new(bar_rect.min.x, bar_rect.max.y),
        );

        crate::ui::color_picker::ShapeColorPicker::new(
            "colorbar_lowclip_picker",
            &mut app.lowclip_color,
            crate::ui::color_picker::ColorShape::LeftTriangle,
        )
        .title("Low Clip Color (< Min)")
        .tooltip("Low Clip color (< Min). Click to select color.")
        .show_at(ui, low_tri_rect);
    }

    if app.use_highclip {
        let high_tri_rect = Rect::from_min_max(
            Pos2::new(bar_rect.max.x, bar_rect.min.y),
            Pos2::new(bar_rect.max.x + tri_w, bar_rect.max.y),
        );

        crate::ui::color_picker::ShapeColorPicker::new(
            "colorbar_highclip_picker",
            &mut app.highclip_color,
            crate::ui::color_picker::ColorShape::RightTriangle,
        )
        .title("High Clip Color (> Max)")
        .tooltip("High Clip color (> Max). Click to select color.")
        .anchor_offset(egui::Vec2::new(-170.0, -250.0))
        .show_at(ui, high_tri_rect);
    }
}
