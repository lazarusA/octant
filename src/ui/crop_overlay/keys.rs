//! The crop box by keyboard: the arrows move it, Shift+arrows move its
//! bottom-right corner to resize it.

use super::handles::{fit_aspect, shift_box};
use crate::export::RoiCropBox;
use egui::{Rect, Vec2};

/// How far one key press moves the box, in points.
const STEP: f32 = 8.0;
/// The smallest side a resize leaves, as a canvas fraction.
const MIN_SIDE: f32 = 0.02;

/// Moves (or with `resize`, resizes) the box one step in `direction` on
/// `canvas_rect`.
pub fn nudge(crop_box: &mut RoiCropBox, direction: Vec2, resize: bool, canvas_rect: Rect) {
    let step = direction * STEP / canvas_rect.size().max(Vec2::splat(1.0));
    if !resize {
        shift_box(crop_box, step.x, step.y);
        return;
    }
    crop_box.u_max = (crop_box.u_max + step.x).clamp(crop_box.u_min + MIN_SIDE, 1.0);
    crop_box.v_max = (crop_box.v_max + step.y).clamp(crop_box.v_min + MIN_SIDE, 1.0);
    fit_aspect(crop_box, canvas_rect);
}
