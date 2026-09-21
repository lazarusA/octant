//! Interactive Region of Interest (ROI) crop overlay and guide lines.

pub mod handles;
pub mod toolbar;

pub use toolbar::CropOverlayAction;

use crate::export::RoiCropBox;
use egui::Rect;
use handles::{draw_crop_handles, handle_crop_interaction};
use toolbar::render_crop_toolbar;

/// Renders the interactive Region of Interest (ROI) crop box and guiding lines on top of the canvas.
pub fn show_crop_overlay(
    ui: &mut egui::Ui,
    canvas_rect: Rect,
    crop_box: &mut RoiCropBox,
    is_open: &mut bool,
) -> Option<CropOverlayAction> {
    if !*is_open || canvas_rect.width() <= 10.0 || canvas_rect.height() <= 10.0 {
        return None;
    }

    crop_box.clamp_bounds();
    let dark_mode = ui.visuals().dark_mode;

    let mask_color = if dark_mode {
        egui::Color32::from_black_alpha(150)
    } else {
        egui::Color32::from_black_alpha(80)
    };

    let accent_color = if dark_mode {
        egui::Color32::from_rgb(0, 190, 255)
    } else {
        egui::Color32::from_rgb(0, 125, 220)
    };

    let grid_stroke = egui::Stroke::new(
        1.0,
        if dark_mode {
            egui::Color32::from_white_alpha(60)
        } else {
            egui::Color32::from_black_alpha(50)
        },
    );

    let border_stroke = egui::Stroke::new(1.5, accent_color);

    let rect_min = egui::pos2(
        canvas_rect.left() + crop_box.u_min * canvas_rect.width(),
        canvas_rect.top() + crop_box.v_min * canvas_rect.height(),
    );
    let rect_max = egui::pos2(
        canvas_rect.left() + crop_box.u_max * canvas_rect.width(),
        canvas_rect.top() + crop_box.v_max * canvas_rect.height(),
    );

    let box_rect = egui::Rect::from_min_max(rect_min, rect_max);
    let painter = ui.painter().with_clip_rect(canvas_rect);

    // 1. Dimmed outer mask (4 surrounding rectangles)
    painter.rect_filled(
        egui::Rect::from_min_max(
            canvas_rect.min,
            egui::pos2(canvas_rect.right(), box_rect.top()),
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.bottom()),
            canvas_rect.max,
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.top()),
            egui::pos2(box_rect.left(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(box_rect.right(), box_rect.top()),
            egui::pos2(canvas_rect.right(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );

    // 2. Rule-of-Thirds Grid Lines
    let w_third = box_rect.width() / 3.0;
    let h_third = box_rect.height() / 3.0;

    painter.line_segment(
        [
            egui::pos2(box_rect.left() + w_third, box_rect.top()),
            egui::pos2(box_rect.left() + w_third, box_rect.bottom()),
        ],
        grid_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(box_rect.left() + 2.0 * w_third, box_rect.top()),
            egui::pos2(box_rect.left() + 2.0 * w_third, box_rect.bottom()),
        ],
        grid_stroke,
    );

    painter.line_segment(
        [
            egui::pos2(box_rect.left(), box_rect.top() + h_third),
            egui::pos2(box_rect.right(), box_rect.top() + h_third),
        ],
        grid_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(box_rect.left(), box_rect.top() + 2.0 * h_third),
            egui::pos2(box_rect.right(), box_rect.top() + 2.0 * h_third),
        ],
        grid_stroke,
    );

    // 3. Boundary Border
    painter.rect_stroke(box_rect, 0.0, border_stroke, egui::StrokeKind::Outside);

    // 4. Interactive Drag & Resize Handling
    handle_crop_interaction(ui, canvas_rect, box_rect, crop_box);

    // 5. Handles
    draw_crop_handles(&painter, box_rect, accent_color, dark_mode);

    // 6. Floating Toolbar
    render_crop_toolbar(ui, canvas_rect, box_rect, crop_box, is_open, accent_color)
}
