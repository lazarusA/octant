//! Double slider widget with numeric drag inputs.

use egui::{DragValue, Id, Rect, Sense, Stroke, Ui, Vec2, pos2};

const HANDLE_RADIUS: f32 = 6.0;

/// Width of the right-hand value box on every dimension row, so the boxes
/// and slider tracks end at the same x on every row. Fits 7 digits.
pub const VALUE_BOX_W: f32 = 56.0;

/// Double slider with numeric input fields on both sides.
///
/// Laid out left to right (start box, track, fixed-width end box): the track
/// fills the space between, and the handles update `end` before the end box
/// is drawn, so the box never lags behind a drag.
pub fn double_slider_with_inputs(
    ui: &mut Ui,
    id_source: impl egui::AsIdSalt,
    start: &mut usize,
    end: &mut usize,
    min: usize,
    max: usize,
) -> bool {
    let mut changed = false;
    let base_id = ui.id().with("double_slider").with(id_source);

    ui.horizontal(|ui| {
        changed |= ui
            .push_id(base_id.with("start_input"), |ui| {
                ui.add(DragValue::new(start).range(min..=*end).speed(1))
            })
            .inner
            .changed();

        let spacing = ui.spacing().item_spacing.x;
        let track_width = (ui.available_width() - VALUE_BOX_W - spacing).max(40.0);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(track_width, 2.0 * HANDLE_RADIUS + 4.0),
            Sense::hover(),
        );
        changed |= range_track(ui, rect, base_id, start, end, min, max);

        changed |= ui
            .push_id(base_id.with("end_input"), |ui| {
                let height = ui.spacing().interact_size.y;
                ui.add_sized(
                    [VALUE_BOX_W, height],
                    DragValue::new(end).range(*start..=max).speed(1),
                )
            })
            .inner
            .changed();
    });

    *start = (*start).clamp(min, max);
    *end = (*end).clamp(min, max).max(*start);

    changed
}

/// Paint the track and selected range in `rect`, and drag both handles.
fn range_track(
    ui: &Ui,
    rect: Rect,
    base_id: Id,
    start: &mut usize,
    end: &mut usize,
    min: usize,
    max: usize,
) -> bool {
    let span = max.saturating_sub(min).max(1) as f32;
    let left = rect.left() + HANDLE_RADIUS;
    let right = rect.right() - HANDLE_RADIUS;
    let mid_y = rect.center().y;
    let to_x = |v: usize| left + (v.saturating_sub(min) as f32 / span) * (right - left);
    let from_x = |x: f32| {
        let t = ((x - left) / (right - left)).clamp(0.0, 1.0);
        min + (t * span).round() as usize
    };

    let painter = ui.painter_at(rect);
    painter.line_segment(
        [pos2(left, mid_y), pos2(right, mid_y)],
        Stroke::new(2.0, ui.visuals().widgets.inactive.bg_fill),
    );
    painter.line_segment(
        [pos2(to_x(*start), mid_y), pos2(to_x(*end), mid_y)],
        Stroke::new(4.0, ui.visuals().selection.bg_fill),
    );

    // Each handle is clamped against the other so they never cross.
    let lo = *start;
    let hi = *end;
    let mut changed = drag_handle(ui, base_id.with("start_handle"), to_x(lo), mid_y, |x| {
        *start = from_x(x).min(hi);
    });
    changed |= drag_handle(ui, base_id.with("end_handle"), to_x(hi), mid_y, |x| {
        *end = from_x(x).max(*start);
    });
    changed && (*start != lo || *end != hi)
}

/// One draggable round handle at (`x`, `y`). Calls `on_drag` with the pointer
/// x while dragged and returns whether it was dragged this frame.
fn drag_handle(ui: &Ui, id: Id, x: f32, y: f32, on_drag: impl FnOnce(f32)) -> bool {
    let rect = Rect::from_center_size(pos2(x, y), Vec2::splat(2.0 * HANDLE_RADIUS));
    let response = ui.interact(rect, id, Sense::drag());
    let dragged_to = response
        .dragged()
        .then(|| response.interact_pointer_pos())
        .flatten();
    if let Some(pos) = dragged_to {
        on_drag(pos.x);
    }
    ui.painter().circle(
        pos2(x, y),
        HANDLE_RADIUS,
        ui.visuals().widgets.inactive.bg_fill,
        ui.style().interact(&response).fg_stroke,
    );
    dragged_to.is_some()
}
