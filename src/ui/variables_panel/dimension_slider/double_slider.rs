//! Double slider widget with numeric drag inputs.

use egui::{DragValue, Sense, Stroke, Ui, Vec2};

/// Double slider with numeric input fields on both sides.
pub fn double_slider_with_inputs(
    ui: &mut Ui,
    id_source: impl egui::AsIdSalt,
    start: &mut usize,
    end: &mut usize,
    min: usize,
    max: usize,
) -> bool {
    let mut changed = false;
    let handle_radius: f32 = 6.0;
    let base_id = ui.id().with("double_slider").with(id_source);

    ui.horizontal(|ui| {
        changed |= ui
            .push_id(base_id.with("start_input"), |ui| {
                ui.add(DragValue::new(start).range(min..=*end).speed(1))
            })
            .inner
            .changed();

        // Right-to-left: the end input pins to the right edge, then the track
        // takes the remaining width, so every row ends at the same x.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed |= ui
                .push_id(base_id.with("end_input"), |ui| {
                    ui.add(DragValue::new(end).range(*start..=max).speed(1))
                })
                .inner
                .changed();

            // The track fills whatever the right-aligned end input leaves.
            let track_width = ui.available_width().max(40.0);
            let (rect, _resp) = ui.allocate_exact_size(
                Vec2::new(track_width, 2.0 * handle_radius + 4.0),
                Sense::hover(),
            );

            let span = max.saturating_sub(min).max(1) as f32;
            let left = rect.left() + handle_radius;
            let right = rect.right() - handle_radius;

            let to_x = |v: usize| left + ((v - min) as f32 / span) * (right - left);
            let from_x = |x: f32| {
                let t = ((x - left) / (right - left)).clamp(0.0, 1.0);
                min + (t * span).round() as usize
            };

            let painter = ui.painter_at(rect);
            let mid_y = rect.center().y;

            painter.line_segment(
                [egui::pos2(left, mid_y), egui::pos2(right, mid_y)],
                Stroke::new(2.0, ui.visuals().widgets.inactive.bg_fill),
            );

            let x0 = to_x(*start);
            let x1 = to_x(*end);

            painter.line_segment(
                [egui::pos2(x0, mid_y), egui::pos2(x1, mid_y)],
                Stroke::new(4.0, ui.visuals().selection.bg_fill),
            );

            let start_rect = egui::Rect::from_center_size(
                egui::pos2(x0, mid_y),
                Vec2::splat(2.0 * handle_radius),
            );
            let start_resp = ui.interact(start_rect, base_id.with("start_handle"), Sense::drag());
            if let Some(pos) = start_resp
                .dragged()
                .then(|| start_resp.interact_pointer_pos())
                .flatten()
            {
                let v = from_x(pos.x).min(*end);
                if v != *start {
                    *start = v;
                    changed = true;
                }
            }
            painter.circle(
                egui::pos2(x0, mid_y),
                handle_radius,
                ui.visuals().widgets.inactive.bg_fill,
                ui.style().interact(&start_resp).fg_stroke,
            );

            let end_rect = egui::Rect::from_center_size(
                egui::pos2(x1, mid_y),
                Vec2::splat(2.0 * handle_radius),
            );
            let end_resp = ui.interact(end_rect, base_id.with("end_handle"), Sense::drag());
            if let Some(pos) = end_resp
                .dragged()
                .then(|| end_resp.interact_pointer_pos())
                .flatten()
            {
                let v = from_x(pos.x).max(*start);
                if v != *end {
                    *end = v;
                    changed = true;
                }
            }
            painter.circle(
                egui::pos2(x1, mid_y),
                handle_radius,
                ui.visuals().widgets.inactive.bg_fill,
                ui.style().interact(&end_resp).fg_stroke,
            );
        });
    });

    *start = (*start).clamp(min, max);
    *end = (*end).clamp(min, max).max(*start);

    changed
}
