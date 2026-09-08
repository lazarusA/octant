//! 1D Line plot series lookup, inverse projection, and guideline painting.

use crate::app::OctantApp;
use egui::{Context, Pos2, Rect, Stroke, Ui};

/// Computes normalized `(nx, ny)` coordinates from screen position for 1D line charts.
pub fn screen_to_norm_1d(app: &OctantApp, rect: Rect, hover_pos: Pos2) -> (f32, f32) {
    let zoom = app.line_zoom;
    let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
    let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

    let ndc_x = ((hover_pos.x - rect.min.x) / rect.width().max(1.0)) * 2.0 - 1.0;
    let unpanned_x = (ndc_x - gpu_pan_x) / zoom.max(0.01);
    let nx = ((unpanned_x + 1.0) / 2.0).clamp(0.0, 1.0);

    let ndc_y = 1.0 - ((hover_pos.y - rect.min.y) / rect.height().max(1.0)) * 2.0;
    let unpanned_y = (ndc_y - gpu_pan_y) / zoom.max(0.01);
    let ny = ((unpanned_y + 1.0) / 2.0).clamp(0.0, 1.0);

    (nx, ny)
}

/// Finds the closest line series value at the sampled X location.
/// Returns `(sample_idx, best_line_idx, val)`.
pub fn sample_line_series(
    app: &OctantApp,
    norm_x: f32,
    norm_y: f32,
    profile_values: &[f32],
    prof_len: usize,
    l_count: usize,
) -> (usize, usize, f32) {
    let sample_idx = if prof_len > 1 {
        ((norm_x * (prof_len - 1) as f32) + 0.5) as usize
    } else {
        0
    }
    .min(prof_len.saturating_sub(1));

    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);

    let mut best_line_idx = 0usize;
    let mut best_dist = f32::INFINITY;
    let mut best_val = f32::NAN;

    if l_count > 0 {
        for line_idx in 0..l_count {
            let idx = line_idx * prof_len + sample_idx;
            if let Some(&v) = profile_values.get(idx)
                && !v.is_nan()
                && v.is_finite()
            {
                let norm_y_val = (((v - cmin) / range) * 2.0 - 1.0).clamp(-1.0, 1.0);
                let dist = (norm_y_val - (norm_y * 2.0 - 1.0)).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_line_idx = line_idx;
                    best_val = v;
                }
            }
        }
    }

    let val = if !best_val.is_nan() {
        best_val
    } else {
        profile_values.get(sample_idx).copied().unwrap_or(f32::NAN)
    };

    (sample_idx, best_line_idx, val)
}

/// Draws guidelines and reticle marker dot for 1D line plots.
pub fn draw_line_guidelines_and_reticle(
    app: &OctantApp,
    ctx: &Context,
    ui: &mut Ui,
    rect: Rect,
    px: usize,
    raw_val: f32,
) {
    let (_, profile_length, _) = app.get_line_profile_payload();
    let prof_len = profile_length as usize;
    let norm_x_step = if prof_len > 1 {
        px as f32 / (prof_len - 1) as f32
    } else {
        0.5
    };

    let zoom = app.line_zoom;
    let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
    let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

    let ndc_x = (norm_x_step * 2.0 - 1.0) * zoom + gpu_pan_x;
    let screen_dot_x = rect.min.x + (ndc_x + 1.0) * 0.5 * rect.width();

    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);

    let norm_y_val = if !raw_val.is_nan() && raw_val.is_finite() {
        ((raw_val - cmin) / range).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let ndc_y = (norm_y_val * 2.0 - 1.0) * zoom + gpu_pan_y;
    let screen_dot_y = rect.min.y + (1.0 - ndc_y) * 0.5 * rect.height();

    let marker_pos = Pos2::new(screen_dot_x, screen_dot_y);

    let x_start_ndc = -zoom + gpu_pan_x;
    let x_end_ndc = 1.0 * zoom + gpu_pan_x;
    let y_top_ndc = 1.0 * zoom + gpu_pan_y;
    let y_bottom_ndc = -zoom + gpu_pan_y;

    let x_line_start = rect.min.x + ((x_start_ndc + 1.0) / 2.0) * rect.width();
    let x_line_end = rect.min.x + ((x_end_ndc + 1.0) / 2.0) * rect.width();
    let y_line_top = rect.min.y + ((1.0 - y_top_ndc) / 2.0) * rect.height();
    let y_line_bottom = rect.min.y + ((1.0 - y_bottom_ndc) / 2.0) * rect.height();

    let x_axis_min = x_line_start.min(x_line_end).clamp(rect.min.x, rect.max.x);
    let x_axis_max = x_line_start.max(x_line_end).clamp(rect.min.x, rect.max.x);
    let y_axis_min = y_line_top.min(y_line_bottom).clamp(rect.min.y, rect.max.y);
    let y_axis_max = y_line_top.max(y_line_bottom).clamp(rect.min.y, rect.max.y);

    let visuals = &ctx.style_of(ctx.theme()).visuals;
    let strong_color = visuals.strong_text_color();
    let text_color = visuals.text_color();
    let line_color = visuals.widgets.noninteractive.fg_stroke.color;

    let painter = ui.painter();

    if marker_pos.x >= rect.min.x && marker_pos.x <= rect.max.x {
        painter.line_segment(
            [
                Pos2::new(marker_pos.x, y_axis_min),
                Pos2::new(marker_pos.x, y_axis_max),
            ],
            Stroke::new(1.0, line_color.linear_multiply(0.7)),
        );
    }

    if marker_pos.y >= rect.min.y && marker_pos.y <= rect.max.y {
        painter.line_segment(
            [
                Pos2::new(x_axis_min, marker_pos.y),
                Pos2::new(x_axis_max, marker_pos.y),
            ],
            Stroke::new(1.0, line_color.linear_multiply(0.7)),
        );
    }

    if rect.contains(marker_pos) {
        painter.circle_filled(marker_pos, 8.0, text_color.linear_multiply(0.12));
        painter.circle_filled(marker_pos, 5.0, text_color.linear_multiply(0.25));
        painter.circle_stroke(marker_pos, 4.0, Stroke::new(1.5, strong_color));
        painter.circle_filled(marker_pos, 2.0, strong_color);
    }
}
